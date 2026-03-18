use crate::riscv::assembler;
use crate::riscv::cpu;
use crate::riscv::memory;
use crate::riscv::register;
use std::cell::RefCell;
use std::fs;
use std::io;
use std::num::ParseIntError;
use std::path::Path;
use std::rc::Rc;

#[derive(Clone, Copy)]
pub struct VMConfig {
    pub(crate) memory_size: memory::Address128,
    pub(crate) debug_mode: bool,
    pub(crate) trace_enabled: bool,
    max_cycles: u64,
}

pub struct VirtualMachine {
    memory: Rc<RefCell<memory::Memory>>,
    cpu: Option<cpu::CPU>,
    config: VMConfig,
    initialized: bool,
}

impl VMConfig {
    pub fn new() -> VMConfig {
        VMConfig {
            memory_size: memory::Memory::DEFAULT_SIZE,
            debug_mode: false,
            trace_enabled: false,
            max_cycles: 0,
        }
    }
}

impl VirtualMachine {
    pub fn new(config: VMConfig) -> Self {
        VirtualMachine {
            memory: Rc::new(RefCell::new(memory::Memory::new(config.memory_size))),
            cpu: None,
            config: config.clone(),
            initialized: false,
        }
    }

    pub fn initialize(&mut self) -> bool {
        self.cpu = Some(cpu::CPU::new(Rc::clone(&self.memory)));

        if let Some(ref mut cpu) = self.cpu {
            cpu.set_debug_mode(self.config.debug_mode);
        } else {
            eprintln!("Initialization failed");
            self.initialized = false;
            return false;
        }

        self.initialized = true;
        true
    }

    pub fn reset(&mut self) {
        if self.initialized {
            if let Some(ref mut cpu) = self.cpu {
                cpu.reset();
            }
            self.memory.borrow_mut().reset();
        }
    }

    pub fn shutdown(&mut self) {
        if self.initialized {
            self.halt();
            self.initialized = false;
        }
    }

    pub fn load_program(&mut self, file_name: &str, load_address: memory::Address128) -> io::Result<bool> {
        if !self.initialized {
            eprintln!("VM not initialized");
            return Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "Virtual machine not initialized"
            ));
        }

        let buffer = fs::read(file_name)?;

        Ok(self.load_bytes(&buffer, buffer.len(), load_address))
    }


    pub fn load_bytes(&mut self, data: &[u8], size: usize, load_address: memory::Address128) -> bool {
        if !self.initialized {
            eprintln!("VM not initialized");
            return false;
        }

        self.memory.borrow_mut().write_bytes(load_address, data, size);

        if let Some(ref mut cpu) = self.cpu {
            cpu.get_registers_mut().set_pc(load_address);
        }

        true
    }

    pub fn load_hex(&mut self, hex_data: &str, load_address: memory::Address128) -> Result<bool, ParseIntError> {
        let data: Vec<u8> = hex_data
            .as_bytes()
            .chunks(2)
            .map(|chunk| {
                let byte_str = std::str::from_utf8(chunk).unwrap_or("00");
                u8::from_str_radix(byte_str, 16)
            })
            .collect::<Result<Vec<u8>, _>>()?;

        Ok(self.load_bytes(&data, data.len(), load_address))
    }

    pub fn load_assembly(&mut self, file_name: &str, load_address: memory::Address128) -> Result<bool, assembler::AssemblyError> {
        if !self.initialized {
            return Err(assembler::AssemblyError {
                line: 0,
                message: "Virtual machine not initialized".to_string(),
            });
        }

        let addr = load_address as u64;
        let machine_code = assembler::assemble_file(file_name, addr)?;
        
        Ok(self.load_bytes(&machine_code, machine_code.len(), load_address))
    }

    pub fn load_assembly_string(&mut self, source: &str, load_address: memory::Address128) -> Result<bool, assembler::AssemblyError> {
        if !self.initialized {
            return Err(assembler::AssemblyError {
                line: 0,
                message: "Virtual machine not initialized".to_string(),
            });
        }

        let addr = load_address as u64;
        let machine_code = assembler::assemble_string(source, addr)?;
        
        Ok(self.load_bytes(&machine_code, machine_code.len(), load_address))
    }

    pub fn is_assembly_file(file_name: &str) -> bool {
        let path = Path::new(file_name);
        if let Some(ext) = path.extension() {
            matches!(ext.to_str(), Some("s") | Some("S") | Some("asm") | Some("ASM"))
        } else {
            false
        }
    }

    pub fn run(&mut self) {
        if !self.initialized {
            eprintln!("VM not initialized");
            return;
        }

        if let Some(ref mut cpu) = self.cpu {
            cpu.run();
        }
    }

    pub fn step(&mut self) {
        if !self.initialized {
            eprintln!("VM not initialized");
            return;
        }

        if let Some(ref mut cpu) = self.cpu {
            cpu.step();

            if self.config.trace_enabled {
                println!("Step completed. PC: 0x{:x}", cpu.get_registers().get_pc())
            }
        }
    }

    pub fn halt(&mut self) {
        if self.initialized {
            if let Some(ref mut cpu) = self.cpu {
                cpu.halt();
            }
        }
    }

    pub fn is_running(&self) -> bool {
        self.initialized && self.cpu.as_ref().unwrap().is_running()
    }

    pub fn start(&mut self) {
        if self.initialized {
            if let Some(ref mut cpu) = self.cpu {
                cpu.start();
            }
        }
    }

    pub fn is_halted(&self) -> bool {
        !self.initialized || self.cpu.as_ref().unwrap().is_halted()
    }

    pub fn has_exception(&self) -> bool {
        self.initialized && self.cpu.as_ref().unwrap().get_state() == cpu::CPUState::Exception
    }

    pub fn get_cpu_state(&self) -> cpu::CPUState {
        if self.initialized { self.cpu.as_ref().unwrap().get_state() } else { cpu::CPUState::Halted }
    }

    pub fn get_cycle_count(&self) -> u64 {
        if self.initialized { self.cpu.as_ref().unwrap().get_cycle_count() } else { 0 }
    }

    pub fn get_last_exception(&self) -> Option<cpu::Exception> {
        if self.initialized {
            self.cpu.as_ref().and_then(|cpu| cpu.get_last_exception().map(|e| e.clone()))
        } else {
            None
        }
    }

    pub fn get_cpu(&self) -> Option<&cpu::CPU> {
        if self.initialized {
            self.cpu.as_ref()
        } else {
            None
        }
    }

    pub fn get_cpu_mut(&mut self) -> Option<&mut cpu::CPU> {
        if self.initialized {
            self.cpu.as_mut()
        } else {
            None
        }
    }

    pub fn get_memory(&self) -> Option<Rc<RefCell<memory::Memory>>> {
        if self.initialized {
            Some(Rc::clone(&self.memory))
        } else {
            None
        }
    }

    pub fn set_debug_mode(&mut self, debug_mode: bool) {
        self.config.debug_mode = debug_mode;
        if self.initialized {
            if let Some(ref mut cpu) = self.cpu {
                cpu.set_debug_mode(debug_mode);
            }
        }
    }

    pub fn set_trace_enabled(&mut self, trace_enabled: bool) {
        self.config.trace_enabled = trace_enabled;
    }

    pub fn print_register_state(&self) {
        if !self.initialized {
            eprintln!("VM not initialized");
            return;
        }

        let mut regs = &register::Register::new();

        if let Some(ref cpu) = self.cpu {
            regs = cpu.get_registers();
        }
        println!("Register state:");
        println!("  PC: 0x{:x}", regs.get_pc());

        for i in 0..32 {
            let reg_index = unsafe { std::mem::transmute::<u8, register::RegisterIndex>(i as u8) };
            let reg_value = regs.read(reg_index);

            let hex_str = format!("{:032x}", reg_value as u128);

            print!("  x{:02}: 0x{}", i, hex_str);

            if (i + 1) % 4 == 0 {
                println!();
            } else {
                print!("  ");
            }
        }
    }

    pub fn print_memory_range(&self, start: memory::Address128, size: usize) {
        if !self.initialized {
            eprintln!("VM not initialized");
            return;
        }

        println!("Memory Range [0x{:x} - 0x{:x}]: ", start, start + size as u128);

        let memory_ref = self.memory.borrow();

        for i in (0..size).step_by(16) {
            let address = start.wrapping_add(i as u128);
            print!("  0x{:032x}: ", address);

            for j in 0..16 {
                if i + j < size {
                    let byte = memory_ref.read_8(start.wrapping_add((i + j) as u128));
                    print!("{:02x}", byte);
                    if (j + 1) % 4 == 0 {
                        print!(" ");
                    }
                }
            }
            println!();
        }
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    pub fn get_config(&self) -> &VMConfig {
        &self.config
    }
}