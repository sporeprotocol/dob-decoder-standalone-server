// refer to https://github.com/nervosnetwork/ckb-vm/blob/develop/examples/ckb-vm-runner.rs

use std::collections::HashMap;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

use ckb_types::packed::{CellOutput, OutPoint};
use ckb_types::prelude::Entity;
use ckb_types::H256;
use ckb_vm::cost_model::estimate_cycles;
use ckb_vm::registers::{A0, A1, A2, A3, A7};
use ckb_vm::{Bytes, CoreMachine, Memory, Register, SupportMachine, Syscalls};

use crate::client::RPC;
use crate::decoder::helpers::extract_dob_information;
use crate::types::{Error, Settings};

macro_rules! error {
    ($err: expr) => {{
        let error = $err.to_string();
        #[cfg(test)]
        println!("[ERROR] {error}");
        #[cfg(not(test))]
        jsonrpsee::tracing::error!("{error}");
        ckb_vm::error::Error::Unexpected(error)
    }};
}

enum DobRingPointer {
    OutPoint(OutPoint),
    TypeHash([u8; 32]),
}

struct DebugSyscall {
    output: Arc<Mutex<Vec<String>>>,
}

impl<Mac: SupportMachine> Syscalls<Mac> for DebugSyscall {
    fn initialize(&mut self, _machine: &mut Mac) -> Result<(), ckb_vm::error::Error> {
        Ok(())
    }

    fn ecall(&mut self, machine: &mut Mac) -> Result<bool, ckb_vm::error::Error> {
        let code = &machine.registers()[A7];
        if code.to_i32() != 2177 {
            return Ok(false);
        }

        let mut addr = machine.registers()[A0].to_u64();
        let mut buffer = Vec::new();

        loop {
            let byte = machine
                .memory_mut()
                .load8(&Mac::REG::from_u64(addr))?
                .to_u8();
            if byte == 0 {
                break;
            }
            buffer.push(byte);
            addr += 1;
        }

        self.output
            .clone()
            .lock()
            .unwrap()
            .push(String::from_utf8(buffer).unwrap());

        Ok(true)
    }
}

struct DobRingMatchSyscall<T: RPC + 'static> {
    ckb_rpc: T,
    ring_tail_confirmation_type_hash: [u8; 32],
    cluster_dnas: HashMap<[u8; 32], Vec<String>>,
    protocol_versions: Vec<String>,
}

impl<T: RPC> DobRingMatchSyscall<T> {
    fn update_dob_ring_cluster_dnas(&mut self, mut out_point: OutPoint) -> Result<(), Error> {
        let (tx, rx) = mpsc::channel();
        let ckb_rpc = self.ckb_rpc.clone();
        let confirmation_type_hash = self.ring_tail_confirmation_type_hash;
        let protocol_versions = self.protocol_versions.clone();
        let mut cluster_dnas = HashMap::<[u8; 32], Vec<String>>::new();
        thread::spawn(move || loop {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let dob_cell = match rt.block_on(ckb_rpc.get_live_cell(&out_point.into(), true)) {
                Ok(cell) => cell,
                Err(err) => {
                    return tx.send(Err(err)).expect("send");
                }
            };
            // extract every single dob information in the ring
            let (cluster_id, dna, next_outpoint) = if let Some(cell) = dob_cell.cell {
                let dob_output = CellOutput::from(cell.output);
                let args = dob_output.lock().args().raw_data();
                let ring_pointer = match OutPoint::from_compatible_slice(&args) {
                    Ok(out_point) => DobRingPointer::OutPoint(out_point),
                    Err(_) => {
                        if args.len() == 32 {
                            DobRingPointer::TypeHash(args.to_vec().try_into().unwrap())
                        } else {
                            return tx
                                .send(Err(Error::InvalidNextDobRingPointer))
                                .expect("send");
                        }
                    }
                };
                let Some(spore_type) = dob_output.type_().to_opt() else {
                    return tx.send(Err(Error::InvalidDOBCell)).expect("send");
                };
                let ((_, dna), cluster_id, _) = match extract_dob_information(
                    cell.data.unwrap().content.as_bytes(),
                    spore_type,
                    &protocol_versions,
                ) {
                    Ok(info) => info,
                    Err(err) => {
                        return tx.send(Err(err)).expect("send");
                    }
                };
                (cluster_id, dna, ring_pointer)
            } else {
                return tx.send(Err(Error::CellOutputNotFound)).expect("send");
            };
            // record cluster and dna
            cluster_dnas.entry(cluster_id).or_default().push(dna);
            // check ring pointer
            match next_outpoint {
                DobRingPointer::OutPoint(next_outpoint) => {
                    out_point = next_outpoint;
                }
                DobRingPointer::TypeHash(type_hash) => {
                    if type_hash != confirmation_type_hash {
                        return tx.send(Err(Error::DobRingUncirclelized)).expect("send");
                    } else {
                        tx.send(Ok(cluster_dnas)).expect("send");
                        break;
                    }
                }
            }
        });
        self.cluster_dnas = rx.recv().expect("recv")?;
        println!("cluster_dnas = {:?}", self.cluster_dnas);
        Ok(())
    }

    fn handle_cluster_type_hash<Mac: SupportMachine>(
        &self,
        machine: &mut Mac,
        buffer_addr: u64,
        buffer_size_addr: &<Mac as CoreMachine>::REG,
        buffer_size: u64,
        cluster_type_hash: &[u8; 32],
    ) -> Result<bool, ckb_vm::error::Error> {
        println!("handle");
        if let Some(dnas) = self.cluster_dnas.get(cluster_type_hash) {
            println!("dnas.len = {}", dnas.len());
            let dna_stream = dnas.join("|");
            machine.memory_mut().store64(
                buffer_size_addr,
                &Mac::REG::from_u64(dna_stream.len() as u64),
            )?;
            if buffer_size > 0 {
                // return real result
                machine
                    .memory_mut()
                    .store_bytes(buffer_addr, &dna_stream.as_bytes()[..buffer_size as usize])?;
            }
            Ok(true)
        } else if !self.cluster_dnas.is_empty() {
            // this branch means the cluster type_hash has missed out
            machine
                .memory_mut()
                .store64(buffer_size_addr, &Mac::REG::from_u64(0))?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl<Mac: SupportMachine, T: RPC> Syscalls<Mac> for DobRingMatchSyscall<T> {
    fn initialize(&mut self, _machine: &mut Mac) -> Result<(), ckb_vm::error::Error> {
        Ok(())
    }

    fn ecall(&mut self, machine: &mut Mac) -> Result<bool, ckb_vm::error::Error> {
        let code = &machine.registers()[A7];
        if code.to_i32() != 2077 {
            return Ok(false);
        }

        // prepare input arguments
        let buffer_addr = machine.registers()[A0].to_u64();
        let buffer_size_addr = machine.registers()[A1].clone();
        let buffer_size = machine.memory_mut().load64(&buffer_size_addr)?.to_u64();
        let outpoint_addr = machine.registers()[A2].to_u64();
        let cluster_type_hash_addr = machine.registers()[A3].to_u64();

        // extract cluster type_hash from addr
        let cluster_type_hash_bytes = machine
            .memory_mut()
            .load_bytes(cluster_type_hash_addr, 32)?;
        let cluster_type_hash: [u8; 32] = cluster_type_hash_bytes.to_vec().try_into().unwrap();

        // checkpoint check for a quick return
        if self.handle_cluster_type_hash(
            machine,
            buffer_addr,
            &buffer_size_addr,
            buffer_size,
            &cluster_type_hash,
        )? {
            return Ok(true);
        }

        // extract outpoint from addr
        let outpoint_bytes = machine
            .memory_mut()
            .load_bytes(outpoint_addr, OutPoint::TOTAL_SIZE as u64)?;
        let ring_tail_outpoint =
            OutPoint::from_compatible_slice(&outpoint_bytes).map_err(|err| error!(err))?;

        // search dob ring that starts from ring_tail_outpoint
        self.update_dob_ring_cluster_dnas(ring_tail_outpoint)
            .map_err(|err| error!(err))?;

        // handle cluster type hash after filling cluster_dnas
        self.handle_cluster_type_hash(
            machine,
            buffer_addr,
            &buffer_size_addr,
            buffer_size,
            &cluster_type_hash,
        )?;

        Ok(true)
    }
}

fn main_asm<T: RPC + 'static>(
    code: Bytes,
    args: Vec<Bytes>,
    type_hash: H256,
    rpc: T,
    settings: &Settings,
) -> Result<(i8, Vec<String>), Box<dyn std::error::Error>> {
    let debug_result = Arc::new(Mutex::new(Vec::new()));
    let debug = Box::new(DebugSyscall {
        output: debug_result.clone(),
    });
    let dob_ring_match = Box::new(DobRingMatchSyscall {
        ckb_rpc: rpc,
        ring_tail_confirmation_type_hash: type_hash.into(),
        cluster_dnas: HashMap::new(),
        protocol_versions: settings.protocol_versions.clone(),
    });

    let asm_core = ckb_vm::machine::asm::AsmCoreMachine::new(
        ckb_vm::ISA_IMC | ckb_vm::ISA_B | ckb_vm::ISA_MOP | ckb_vm::ISA_A,
        ckb_vm::machine::VERSION2,
        u64::MAX,
    );
    let core = ckb_vm::DefaultMachineBuilder::new(asm_core)
        .instruction_cycle_func(Box::new(estimate_cycles))
        .syscall(debug)
        .syscall(dob_ring_match)
        .build();
    let mut machine = ckb_vm::machine::asm::AsmMachine::new(core);
    machine.load_program(&code, &args)?;

    let error_code = machine.run()?;
    let result = debug_result.lock().unwrap().clone();
    Ok((error_code, result))
}

pub fn execute_riscv_binary<T: RPC + 'static>(
    binary_path: &str,
    args: Vec<Bytes>,
    spore_type_hash: H256,
    rpc: T,
    settings: &Settings,
) -> Result<(i8, Vec<String>), Box<dyn std::error::Error>> {
    let code = std::fs::read(binary_path)?.into();
    main_asm(code, args, spore_type_hash, rpc, settings)
}
