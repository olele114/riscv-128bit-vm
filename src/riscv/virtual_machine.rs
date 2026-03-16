use crate::riscv::cpu;
use crate::riscv::memory;
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Clone, Copy)]
pub struct VMConfig {
    memory_size: memory::Address128,
    debug_mode: bool,
    trace_enabled: bool,
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
}