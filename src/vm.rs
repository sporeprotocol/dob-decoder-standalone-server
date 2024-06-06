// refer to https://github.com/nervosnetwork/ckb-vm/blob/develop/examples/ckb-vm-runner.rs

use std::sync::{mpsc, Arc, Mutex};
use std::thread;

use ckb_vm::cost_model::estimate_cycles;
use ckb_vm::registers::{A0, A1, A2, A3, A4, A7};
use ckb_vm::{Bytes, Memory, Register, SupportMachine, Syscalls};
use image::load_from_memory;

use crate::client::ImageFetchClient;
use crate::types::Settings;

macro_rules! error {
    ($err: expr) => {{
        let error = $err.to_string();
        println!("[DOB/1 ERROR] {error}");
        ckb_vm::error::Error::Unexpected(error)
    }};
}

// ckb-vm syscall for printing debug information
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

// ckb-vm syscall for image combination
struct ImageCombinationSyscall {
    client: Arc<Mutex<ImageFetchClient>>,
    max_combination: usize,
}

impl<Mac: SupportMachine> Syscalls<Mac> for ImageCombinationSyscall {
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
        let images_uri_array_addr = machine.registers()[A2].to_u64();
        let images_uri_array_count = machine.registers()[A3].to_u64();
        let iamges_uri_array_uint_size = machine.registers()[A4].to_u64();

        // parse all of images uri
        let array_size = images_uri_array_count * iamges_uri_array_uint_size;
        let images_uri_array_bytes = machine
            .memory_mut()
            .load_bytes(images_uri_array_addr, array_size)?;
        let images_uri_array = images_uri_array_bytes
            .chunks_exact(iamges_uri_array_uint_size as usize)
            .map(|uri_bytes| String::from_utf8_lossy(uri_bytes).to_string())
            .collect::<Vec<_>>();

        #[cfg(feature = "render_debug")]
        {
            println!("-------- DOB/1 IMAGES ---------");
            images_uri_array.iter().for_each(|uri| println!("{uri}"));
            println!("-------- DOB/1 IMAGES END ---------\n");
        }

        // fetch images from uri
        let client = self.client.clone();
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let mut client = client.lock().unwrap();
            tx.send(rt.block_on(client.fetch_images(&images_uri_array)))
                .expect("send");
        });
        let mut images = rx
            .recv()
            .expect("recv")
            .map_err(|err| error!(err))?
            .into_iter()
            .map(|image| load_from_memory(&image))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|err| error!(err))?;
        if images.is_empty() {
            return Err(error!("empty images"));
        }
        if images.len() > self.max_combination {
            return Err(error!("exceesive images"));
        }

        // resize images to the maximum
        let mut max_width = 0;
        let mut max_height = 0;
        images.iter().for_each(|image| {
            max_width = max_width.max(image.width());
            max_height = max_height.max(image.height());
        });
        images.iter_mut().for_each(|image| {
            image.resize(max_width, max_height, image::imageops::FilterType::Nearest);
        });

        // combine images into a single one
        let mut combination = images.remove(0);
        if buffer_size == 0 {
            machine.memory_mut().store64(
                &buffer_size_addr,
                &Mac::REG::from_u64(combination.as_bytes().len() as u64),
            )?;
            return Ok(true);
        }
        images.into_iter().for_each(|image| {
            image::imageops::overlay(&mut combination, &image, 0, 0);
        });

        // return output
        let output = combination.as_bytes();
        let buffer_size = buffer_size.min(output.len() as u64);
        machine.memory_mut().store_bytes(buffer_addr, output)?;
        machine
            .memory_mut()
            .store64(&buffer_size_addr, &Mac::REG::from_u64(buffer_size))?;

        Ok(true)
    }
}

fn main_asm(
    code: Bytes,
    args: Vec<Bytes>,
    settings: &Settings,
) -> Result<(i8, Vec<String>), Box<dyn std::error::Error>> {
    let debug_result = Arc::new(Mutex::new(Vec::new()));
    let debug = Box::new(DebugSyscall {
        output: debug_result.clone(),
    });
    let client = ImageFetchClient::new(&settings.image_fetcher_url, settings.dob1_max_cache_size);
    let image = Box::new(ImageCombinationSyscall {
        client: Arc::new(Mutex::new(client)),
        max_combination: settings.dob1_max_combination,
    });

    let asm_core = ckb_vm::machine::asm::AsmCoreMachine::new(
        ckb_vm::ISA_IMC | ckb_vm::ISA_B | ckb_vm::ISA_MOP | ckb_vm::ISA_A,
        ckb_vm::machine::VERSION2,
        u64::MAX,
    );
    let core = ckb_vm::DefaultMachineBuilder::new(asm_core)
        .instruction_cycle_func(Box::new(estimate_cycles))
        .syscall(debug)
        .syscall(image)
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
    settings: &Settings,
) -> Result<(i8, Vec<String>), Box<dyn std::error::Error>> {
    let code = std::fs::read(binary_path)?.into();
    main_asm(code, args, settings)
}
