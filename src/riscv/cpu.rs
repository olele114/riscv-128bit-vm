use crate::riscv::memory;
use crate::riscv::register;


type InstructionCallback = Box<dyn Fn(u32, memory::Address128)>;

#[derive(PartialEq)]
enum CPUState {
    Running,
    Halted,
    Exception,
}

enum ExceptionType {
    None,
    IllegalInstruction,
    MemoryAccess,
    SystemCall,
    Breakpoint,
    Unknown,
}

struct Exception {
    typ: ExceptionType,
    code: u64,
    address: memory::Address128,
    message: String,
}

struct CPU {
    registers: register::Register,
    memory: memory::Memory,
    state: CPUState,
    last_exception: Option<Exception>,
    cycle_count: u64,
    debug_mode: bool,
}

impl Exception {
    pub fn new(typ: ExceptionType, code: u64, address: memory::Address128, message: String) -> Self {
        Self {
            typ,
            code,
            address,
            message,
        }
    }
}

impl CPU {
    pub fn new(memory: memory::Memory) -> Self {
        Self {
            memory,
            state: CPUState::Halted,
            cycle_count: 0,
            registers: register::Register::new(),
            last_exception: None,
            debug_mode: false,
        }
    }

    pub fn step(&mut self) {
        if self.state == CPUState::Halted {
            return;
        }

        todo!();
    }
}