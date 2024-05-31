// refer to https://github.com/nervosnetwork/ckb-vm/blob/develop/examples/ckb-vm-runner.rs

use std::sync::{Arc, Mutex};

use ckb_vm::cost_model::estimate_cycles;
use ckb_vm::registers::{A0, A7};
use ckb_vm::{Bytes, Memory, Register, SupportMachine, Syscalls};

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

fn main_asm(
    code: Bytes,
    args: Vec<Bytes>,
) -> Result<(i8, Vec<String>), Box<dyn std::error::Error>> {
    let debug_result = Arc::new(Mutex::new(Vec::new()));
    let debug = Box::new(DebugSyscall {
        output: debug_result.clone(),
    });

    let asm_core = ckb_vm::machine::asm::AsmCoreMachine::new(
        ckb_vm::ISA_IMC | ckb_vm::ISA_B | ckb_vm::ISA_MOP | ckb_vm::ISA_A,
        ckb_vm::machine::VERSION2,
        u64::MAX,
    );
    let core = ckb_vm::DefaultMachineBuilder::new(asm_core)
        .instruction_cycle_func(Box::new(estimate_cycles))
        .syscall(debug)
        .build();
    let mut machine = ckb_vm::machine::asm::AsmMachine::new(core);
    machine.load_program(&code, &args)?;

    let error_code = machine.run()?;
    let result = debug_result.lock().unwrap().clone();
    Ok((error_code, result))
}

pub fn execute_riscv_binary(
    binary_path: &str,
    args: Vec<Bytes>,
) -> Result<(i8, Vec<String>), Box<dyn std::error::Error>> {
    let code = std::fs::read(binary_path)?.into();
    Ok(main_asm(code, args)?)
}
