use crate::riscv::memory;
use crate::riscv::register;
use crate::riscv::instruction;
use crate::riscv::executor;
use std::rc::Rc;
use std::cell::RefCell;

type InstructionCallback = Box<dyn Fn(u32, memory::Address128)>;

#[derive(PartialEq, Copy, Clone)]
pub enum CPUState {
    Running,
    Halted,
    Exception,
}

#[derive(Copy, Clone, Debug)]
pub enum ExceptionType {
    None,
    IllegalInstruction,
    MemoryAccess,
    SystemCall,
    Breakpoint,
    Unknown,
}

#[derive(Clone)]
pub struct Exception {
    pub(crate) typ: ExceptionType,
    pub(crate) code: u64,
    pub(crate) address: memory::Address128,
    pub(crate) message: String,
}

pub(crate) struct CPU {
    registers: register::Register,
    memory: Rc<RefCell<memory::Memory>>,
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
    pub fn new(memory: Rc<RefCell<memory::Memory>>) -> Self {
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

        let result = executor::Executor::execute(self, &decoded);

        self.cycle_count += 1;

        if !result.success {
            return;
        } else if result.branch_taken {
            self.registers.set_pc(result.next_pc);
        } else {
            self.increment_pc();
        }
    }

    pub fn run(&mut self) {
        self.state = CPUState::Running;
        while self.state == CPUState::Running {
            self.step();
        }
    }

    pub fn start(&mut self) {
        self.state = CPUState::Running;
    }

    pub fn reset(&mut self) {
        self.registers.reset();
        self.memory.borrow_mut().reset();
        self.state = CPUState::Halted;
        self.cycle_count = 0;
        self.clear_exception();
    }

    pub fn halt(&mut self) {
        self.state = CPUState::Halted;
    }

    pub fn set_debug_mode(&mut self, debug_mode: bool) {
        self.debug_mode = debug_mode;
    }

    pub fn get_state(&self) -> CPUState {
        self.state
    }

    pub fn is_running(&self) -> bool {
        self.state == CPUState::Running
    }

    pub fn is_halted(&self) -> bool {
        self.state == CPUState::Halted
    }

    pub fn get_cycle_count(&self) -> u64 {
        self.cycle_count
    }

    pub fn get_last_exception(&self) -> Option<&Exception> {
        self.last_exception.as_ref()
    }

    pub fn fetch_instruction(&self, pc: memory::Address128) -> u32 {
        self.memory.borrow().read_32(pc)
    }

    pub fn get_registers(&self) -> &register::Register {
        &self.registers
    }

    pub fn get_registers_mut(&mut self) -> &mut register::Register {
        &mut self.registers
    }

    pub fn get_memory(&self) -> &Rc<RefCell<memory::Memory>> {
        &self.memory
    }

    pub fn get_memory_mut(&mut self) -> &mut Rc<RefCell<memory::Memory>> {
        &mut self.memory
    }

    pub fn raise_exception(&mut self, typ: ExceptionType, code: u64, address: memory::Address128, message: String) {
        if self.debug_mode {
            eprintln!("Exception: {} at address: {}", message, address);
        }

        self.last_exception = Some(Exception::new(typ, code, address, message));
        self.state = CPUState::Exception;
    }

    pub fn clear_exception(&mut self) {
        self.last_exception = None;
    }

    pub fn increment_pc(&mut self) {
        self.registers.set_pc(self.registers.get_pc() + 4);
    }
}