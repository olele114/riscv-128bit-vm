//! CPU Core Module
//!
//! Implements the RISC-V CPU core with fetch-decode-execute cycle,
//! register management, and exception handling.
//!
//! ---
//!
//! CPU 核心模块
//!
//! 实现 RISC-V CPU 核心，包括取指-解码-执行周期、
//! 寄存器和异常处理。

#![allow(dead_code)]

use crate::riscv::memory;
use crate::riscv::register;
use crate::riscv::instruction;
use crate::riscv::executor;
use std::rc::Rc;
use std::cell::RefCell;

type InstructionCallback = Box<dyn Fn(u32, memory::Address128)>;

/// CPU execution state
///
/// - `Running`: CPU is actively executing instructions
/// - `Halted`: CPU is stopped (initial state)
/// - `Exception`: CPU encountered an error
///
/// ---
///
/// CPU 执行状态
///
/// - `Running`: CPU 正在执行指令
/// - `Halted`: CPU 已停止（初始状态）
/// - `Exception`: CPU 遇到错误
#[derive(PartialEq, Copy, Clone)]
pub enum CPUState {
    Running,
    Halted,
    Exception,
}

/// Exception types for the CPU
///
/// ---
///
/// CPU 异常类型
#[derive(Copy, Clone, Debug)]
pub enum ExceptionType {
    /// No exception (无异常)
    None,
    /// Illegal or unsupported instruction (非法或不支持的指令)
    IllegalInstruction,
    /// Memory access violation (内存访问违规)
    MemoryAccess,
    /// System call (ECALL) (系统调用)
    SystemCall,
    /// Breakpoint (EBREAK) (断点)
    Breakpoint,
    /// Unknown error (未知错误)
    Unknown,
}

/// Exception information
///
/// Contains details about an exception that occurred during execution.
///
/// ---
///
/// 异常信息
///
/// 包含执行期间发生的异常的详细信息。
#[derive(Clone)]
pub struct Exception {
    /// Exception type (异常类型)
    pub(crate) typ: ExceptionType,
    /// Exception code (异常代码)
    pub(crate) code: u64,
    /// Address where exception occurred (发生异常的地址)
    pub(crate) address: memory::Address128,
    /// Human-readable message (可读消息)
    pub(crate) message: String,
}

/// RISC-V CPU Core
///
/// The main CPU implementation that fetches, decodes, and executes
/// RISC-V instructions. Manages registers, memory access, and exceptions.
///
/// ---
///
/// RISC-V CPU 核心
///
/// 主要的 CPU 实现，执行 RISC-V 指令的取指、解码和执行。
/// 管理寄存器、内存访问和异常。
pub(crate) struct CPU {
    registers: register::Register,
    memory: Rc<RefCell<memory::Memory>>,
    state: CPUState,
    last_exception: Option<Exception>,
    cycle_count: u64,
    debug_mode: bool,
}

impl Exception {
    /// Creates a new Exception with the given details.
    ///
    /// ---
    ///
    /// 使用给定详细信息创建新异常。
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
    /// Creates a new CPU with the given memory.
    ///
    /// The CPU starts in halted state.
    ///
    /// ---
    ///
    /// 使用给定内存创建新 CPU。
    ///
    /// CPU 初始为停止状态。
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

    /// Executes one instruction.
    ///
    /// Performs fetch-decode-execute cycle for one instruction.
    /// Only executes if CPU is in Running state.
    ///
    /// ---
    ///
    /// 执行一条指令。
    ///
    /// 对一条指令执行取指-解码-执行周期。
    /// 仅在 CPU 处于 Running 状态时执行。
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

    /// Runs the CPU until halt or exception.
    ///
    /// Sets state to Running and executes instructions in a loop.
    ///
    /// ---
    ///
    /// 运行 CPU 直到停止或异常。
    ///
    /// 将状态设置为 Running 并循环执行指令。
    pub fn run(&mut self) {
        self.state = CPUState::Running;
        while self.state == CPUState::Running {
            self.step();
        }
    }

    /// Starts the CPU execution.
    ///
    /// Sets state to Running without automatically executing instructions.
    ///
    /// ---
    ///
    /// 启动 CPU 执行。
    ///
    /// 将状态设置为 Running，不自动执行指令。
    pub fn start(&mut self) {
        self.state = CPUState::Running;
    }

    /// Resets the CPU to initial state.
    ///
    /// Clears registers, memory, and cycle count.
    ///
    /// ---
    ///
    /// 将 CPU 重置为初始状态。
    ///
    /// 清除寄存器、内存和周期计数。
    pub fn reset(&mut self) {
        self.registers.reset();
        self.memory.borrow_mut().reset();
        self.state = CPUState::Halted;
        self.cycle_count = 0;
        self.clear_exception();
    }

    /// Halts the CPU.
    ///
    /// ---
    ///
    /// 停止 CPU。
    pub fn halt(&mut self) {
        self.state = CPUState::Halted;
    }

    /// Enables or disables debug mode.
    ///
    /// When enabled, prints instruction details during execution.
    ///
    /// ---
    ///
    /// 启用或禁用调试模式。
    ///
    /// 启用时，执行过程中打印指令详细信息。
    pub fn set_debug_mode(&mut self, debug_mode: bool) {
        self.debug_mode = debug_mode;
    }

    /// Returns the current CPU state.
    ///
    /// ---
    ///
    /// 返回当前 CPU 状态。
    pub fn get_state(&self) -> CPUState {
        self.state
    }

    /// Checks if CPU is running.
    ///
    /// ---
    ///
    /// 检查 CPU 是否正在运行。
    pub fn is_running(&self) -> bool {
        self.state == CPUState::Running
    }

    /// Checks if CPU is halted.
    ///
    /// ---
    ///
    /// 检查 CPU 是否已停止。
    pub fn is_halted(&self) -> bool {
        self.state == CPUState::Halted
    }

    /// Returns the number of executed cycles.
    ///
    /// ---
    ///
    /// 返回已执行的周期数。
    pub fn get_cycle_count(&self) -> u64 {
        self.cycle_count
    }

    /// Returns the last exception, if any.
    ///
    /// ---
    ///
    /// 返回最后的异常（如果有）。
    pub fn get_last_exception(&self) -> Option<&Exception> {
        self.last_exception.as_ref()
    }

    /// Fetches a 32-bit instruction from memory.
    ///
    /// ---
    ///
    /// 从内存中取一条 32 位指令。
    pub fn fetch_instruction(&self, pc: memory::Address128) -> u32 {
        self.memory.borrow().read_32(pc)
    }

    /// Returns a reference to the register file.
    ///
    /// ---
    ///
    /// 返回寄存器组的引用。
    pub fn get_registers(&self) -> &register::Register {
        &self.registers
    }

    /// Returns a mutable reference to the register file.
    ///
    /// ---
    ///
    /// 返回寄存器组的可变引用。
    pub fn get_registers_mut(&mut self) -> &mut register::Register {
        &mut self.registers
    }

    /// Returns a reference to the memory.
    ///
    /// ---
    ///
    /// 返回内存的引用。
    pub fn get_memory(&self) -> &Rc<RefCell<memory::Memory>> {
        &self.memory
    }

    /// Returns a mutable reference to the memory.
    ///
    /// ---
    ///
    /// 返回内存的可变引用。
    pub fn get_memory_mut(&mut self) -> &mut Rc<RefCell<memory::Memory>> {
        &mut self.memory
    }

    /// Raises an exception and stops execution.
    ///
    /// ---
    ///
    /// 引发异常并停止执行。
    pub fn raise_exception(&mut self, typ: ExceptionType, code: u64, address: memory::Address128, message: String) {
        if self.debug_mode {
            eprintln!("Exception: {} at address: {}", message, address);
        }

        self.last_exception = Some(Exception::new(typ, code, address, message));
        self.state = CPUState::Exception;
    }

    /// Clears the last exception.
    ///
    /// ---
    ///
    /// 清除最后的异常。
    pub fn clear_exception(&mut self) {
        self.last_exception = None;
    }

    /// Increments the program counter by 4.
    ///
    /// ---
    ///
    /// 将程序计数器增加 4。
    pub fn increment_pc(&mut self) {
        self.registers.set_pc(self.registers.get_pc() + 4);
    }
}