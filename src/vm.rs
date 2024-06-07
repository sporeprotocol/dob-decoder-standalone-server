// refer to https://github.com/nervosnetwork/ckb-vm/blob/develop/examples/ckb-vm-runner.rs

use std::io::Cursor;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

use ckb_vm::cost_model::estimate_cycles;
use ckb_vm::registers::{A0, A1, A2, A3, A7};
use ckb_vm::{Bytes, Memory, Register, SupportMachine, Syscalls};
use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use image::{imageops, load_from_memory, DynamicImage, Pixel, Rgb, RgbaImage};
use jsonrpsee::tracing;
use molecule::prelude::Entity;

use crate::client::ImageFetchClient;
use crate::types::{generated, Error, Settings};

macro_rules! error {
    ($err: expr) => {{
        let error = $err.to_string();
        tracing::error!("{error}");
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
            .push(String::from_utf8(buffer).map_err(|err| error!(err))?);

        Ok(true)
    }
}

// ckb-vm syscall for image combination
struct ImageCombinationSyscall {
    client: Arc<Mutex<ImageFetchClient>>,
    max_combination: usize,
}

impl ImageCombinationSyscall {
    // fetch one image synchronously
    //
    // note: for saving space purpose, we take time to trade memory space through
    //       fetching images one by one
    fn fetch_one_image_sync(&mut self, image_uri: String) -> Result<Vec<u8>, Error> {
        let (tx, rx) = mpsc::channel();
        let client = self.client.clone();
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let mut client = client.lock().unwrap();
            tx.send(rt.block_on(client.fetch_images(&[image_uri])))
                .expect("send");
        });
        Ok(rx.recv().expect("recv")?.remove(0))
    }
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
        let mut buffer_size = machine.memory_mut().load64(&buffer_size_addr)?.to_u64();
        let molecule_addr = machine.registers()[A2].to_u64();
        let molecule_size = machine.registers()[A3].to_u64();

        // parse all of images uri/color/raw
        let pattern_bytes = machine
            .memory_mut()
            .load_bytes(molecule_addr, molecule_size)?;
        let pattern =
            generated::ItemVec::from_compatible_slice(&pattern_bytes).map_err(|err| error!(err))?;
        if pattern.len() > self.max_combination {
            return Err(error!("too many combine operations"));
        }

        // handle DOB/1 pattern
        #[cfg(feature = "render_debug")]
        {
            println!("\n-------- DOB/1 IMAGES ---------");
        }
        let mut combination = image::DynamicImage::new_rgba8(1, 1);
        for item in pattern.into_iter() {
            match item.to_enum() {
                generated::ItemUnion::Color(color) => {
                    let color_code = String::from_utf8_lossy(&color.raw_data()).to_string();
                    #[cfg(feature = "render_debug")]
                    {
                        println!("COLOR => #{color_code}");
                    }
                    let rgb = hex::decode(color_code).map_err(|err| error!(err))?;
                    if rgb.len() != 3 {
                        return Err(error!("invalid color code"));
                    }
                    let pixel = Rgb([rgb[0], rgb[1], rgb[2]]).to_rgba();
                    let image = DynamicImage::ImageRgba8(RgbaImage::from_pixel(1, 1, pixel));
                    overlay_both_images(&mut combination, image);
                }
                generated::ItemUnion::RawImage(raw_image) => {
                    #[cfg(feature = "render_debug")]
                    {
                        println!("IMAGE => (bytes length: {})", raw_image.raw_data().len());
                    }
                    let image =
                        load_from_memory(&raw_image.raw_data()).map_err(|err| error!(err))?;
                    overlay_both_images(&mut combination, image);
                }
                generated::ItemUnion::URI(uri) => {
                    let uri = String::from_utf8_lossy(&uri.raw_data()).to_string();
                    #[cfg(feature = "render_debug")]
                    {
                        println!("FSURI => {uri}");
                    }
                    let raw_image = self.fetch_one_image_sync(uri).map_err(|err| error!(err))?;
                    let image = load_from_memory(&raw_image).map_err(|err| error!(err))?;
                    overlay_both_images(&mut combination, image);
                }
            }
        }
        #[cfg(feature = "render_debug")]
        {
            println!("-------- DOB/1 IMAGES END ---------");
        }

        // return output
        let mut output = Vec::new();
        let cursor = Cursor::new(&mut output);
        let png = PngEncoder::new_with_quality(cursor, CompressionType::Best, FilterType::NoFilter);
        combination
            .write_with_encoder(png)
            .map_err(|err| error!(err))?;
        if buffer_size > 0 {
            buffer_size = buffer_size.min(output.len() as u64);
            machine
                .memory_mut()
                .store_bytes(buffer_addr, &output[..buffer_size as usize])?;
        } else {
            buffer_size = output.len() as u64;
        }
        std::fs::write("nervape.png", &output).unwrap();
        machine
            .memory_mut()
            .store64(&buffer_size_addr, &Mac::REG::from_u64(buffer_size))?;

        Ok(true)
    }
}

fn overlay_both_images(base: &mut DynamicImage, mut overlayer: DynamicImage) {
    let width = base.width().max(overlayer.width());
    let height = base.height().max(overlayer.height());
    if base.width() != width || base.height() != height {
        *base = base.resize(width, height, imageops::FilterType::Nearest);
    }
    if overlayer.width() != width || overlayer.height() != height {
        overlayer = overlayer.resize(width, height, imageops::FilterType::Nearest);
    }
    imageops::overlay(base, &overlayer, 0, 0);
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
