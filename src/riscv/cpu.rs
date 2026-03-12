use crate::riscv::memory;
use crate::riscv::register;
use crate::riscv::instruction;
use crate::riscv::executor;
use crate::riscv::register::Register;

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

pub(crate) struct CPU {
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

        let pc = self.registers.get_pc();

        let instruction = self.fetch_instruction(pc);

        let decoded = instruction::InstructionDecoder::decode(instruction);

        if self.debug_mode {
            println!("[DEBUG] PC: 0x{:x} Instr: 0x{:x} ({})", pc, instruction, instruction::InstructionDecoder::get_instruction_name(&decoded));
        }


    }

    pub fn fetch_instruction(&self, pc: memory::Address128) -> u32 {
        return self.memory.read_32(pc);
    }

    pub fn get_registers(&self) -> &Register {
        &self.registers
    }

    pub fn get_registers_mut(&mut self) -> &mut Register {
        &mut self.registers
    }
}