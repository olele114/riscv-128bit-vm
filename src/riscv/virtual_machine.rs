//! Virtual Machine Interface
//!
//! Provides a high-level interface for the RISC-V virtual machine,
//! including configuration, program loading, and execution control.
//!
//! # Example
//!
//! ```ignore
//! use riscv::virtual_machine::{VirtualMachine, VMConfig};
//!
//! let config = VMConfig::new();
//! let mut vm = VirtualMachine::new(config);
//! vm.initialize();
//! vm.load_program("program.bin", 0x0).unwrap();
//! vm.run();
//! ```
//!
//! ---
//!
//! 虚拟机接口
//!
//! 提供 RISC-V 虚拟机的高级接口，包括配置、程序加载和执行控制。
//!
//! # 示例
//!
//! ```ignore
//! use riscv::virtual_machine::{VirtualMachine, VMConfig};
//!
//! let config = VMConfig::new();
//! let mut vm = VirtualMachine::new(config);
//! vm.initialize();
//! vm.load_program("program.bin", 0x0).unwrap();
//! vm.run();
//! ```

#![allow(dead_code)]

use crate::riscv::assembler;
use crate::riscv::cpu;
use crate::riscv::debugger;
use crate::riscv::memory;
use crate::riscv::register;
use std::cell::RefCell;
use std::fs;
use std::io;
use std::io::{BufRead, Write};
use std::num::ParseIntError;
use std::path::Path;
use std::rc::Rc;

/// Virtual Machine Configuration
///
/// Contains settings for memory size, debug mode, and tracing.
///
/// ---
///
/// 虚拟机配置
///
/// 包含内存大小、调试模式和追踪设置。
#[derive(Clone)]
pub struct VMConfig {
    /// Memory size in bytes (内存大小，单位字节)
    pub memory_size: memory::Address128,
    /// Enable debug output (启用调试输出)
    pub debug_mode: bool,
    /// Enable execution tracing (启用执行追踪)
    pub trace_enabled: bool,
    /// Enable interactive debugger (启用交互式调试器)
    pub debugger_enabled: bool,
    /// Maximum history entries (最大历史记录条目)
    pub history_size: usize,
    max_cycles: u64,
}

/// RISC-V Virtual Machine
///
/// The main virtual machine structure that manages memory, CPU,
/// and provides methods for loading and executing programs.
///
/// ---
///
/// RISC-V 虚拟机
///
/// 主虚拟机结构，管理内存、CPU，并提供加载和执行程序的方法。
pub struct VirtualMachine {
    memory: Rc<RefCell<memory::Memory>>,
    cpu: Option<cpu::CPU>,
    debugger: Option<debugger::InteractiveDebugger>,
    config: VMConfig,
    initialized: bool,
}

impl VMConfig {
    /// Creates a new VMConfig with default values.
    ///
    /// Default: 16MB memory, debug disabled, tracing disabled.
    ///
    /// ---
    ///
    /// 创建具有默认值的新 VMConfig。
    ///
    /// 默认值：16MB 内存，禁用调试，禁用追踪。
    pub fn new() -> VMConfig {
        VMConfig {
            memory_size: memory::Memory::DEFAULT_SIZE,
            debug_mode: false,
            trace_enabled: false,
            debugger_enabled: false,
            history_size: 1000,
            max_cycles: 0,
        }
    }

    /// Enables the interactive debugger.
    ///
    /// ---
    ///
    /// 启用交互式调试器。
    pub fn with_debugger(mut self) -> Self {
        self.debugger_enabled = true;
        self
    }

    /// Sets the history size.
    ///
    /// ---
    ///
    /// 设置历史记录大小。
    pub fn with_history_size(mut self, size: usize) -> Self {
        self.history_size = size;
        self
    }
}

impl VirtualMachine {
    /// Creates a new VirtualMachine with the given configuration.
    ///
    /// Note: Must call `initialize()` before using the VM.
    ///
    /// ---
    ///
    /// 使用给定配置创建新的 VirtualMachine。
    ///
    /// 注意：使用虚拟机前必须调用 `initialize()`。
    pub fn new(config: VMConfig) -> Self {
        let debugger = if config.debugger_enabled {
            Some(debugger::InteractiveDebugger::new(config.history_size))
        } else {
            None
        };
        
        VirtualMachine {
            memory: Rc::new(RefCell::new(memory::Memory::new(config.memory_size))),
            cpu: None,
            debugger,
            config,
            initialized: false,
        }
    }

    /// Initializes the virtual machine.
    ///
    /// Creates the CPU and sets up the memory connection.
    /// Returns `true` on success, `false` on failure.
    ///
    /// ---
    ///
    /// 初始化虚拟机。
    ///
    /// 创建 CPU 并设置内存连接。
    /// 成功返回 `true`，失败返回 `false`。
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

    /// Resets the virtual machine to initial state.
    ///
    /// Clears registers and memory, sets CPU to halted state.
    ///
    /// ---
    ///
    /// 将虚拟机重置为初始状态。
    ///
    /// 清除寄存器和内存，将 CPU 设置为停止状态。
    pub fn reset(&mut self) {
        if self.initialized {
            if let Some(ref mut cpu) = self.cpu {
                cpu.reset();
            }
            self.memory.borrow_mut().reset();
        }
    }

    /// Shuts down the virtual machine.
    ///
    /// Halts execution and marks the VM as uninitialized.
    ///
    /// ---
    ///
    /// 关闭虚拟机。
    ///
    /// 停止执行并将虚拟机标记为未初始化。
    pub fn shutdown(&mut self) {
        if self.initialized {
            self.halt();
            self.initialized = false;
        }
    }

    /// Loads a binary program from a file.
    ///
    /// Reads the file and loads it into memory at the specified address.
    /// Sets the PC to the load address.
    ///
    /// ---
    ///
    /// 从文件加载二进制程序。
    ///
    /// 读取文件并将其加载到指定地址的内存中。
    /// 将 PC 设置为加载地址。
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

    /// Loads raw bytes into memory.
    ///
    /// Writes the data to memory and sets the PC to the load address.
    ///
    /// ---
    ///
    /// 将原始字节加载到内存中。
    ///
    /// 将数据写入内存并将 PC 设置为加载地址。
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

    /// Loads a hexadecimal string into memory.
    ///
    /// Parses hex data (e.g., "01020304") and loads it into memory.
    ///
    /// ---
    ///
    /// 将十六进制字符串加载到内存中。
    ///
    /// 解析十六进制数据（如 "01020304"）并加载到内存中。
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

    /// Loads and assembles an assembly file.
    ///
    /// Reads the assembly file, assembles it to machine code,
    /// and loads it into memory.
    ///
    /// ---
    ///
    /// 加载并汇编汇编文件。
    ///
    /// 读取汇编文件，将其汇编为机器码，并加载到内存中。
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

    /// Assembles and loads assembly source code from a string.
    ///
    /// Useful for testing or embedding assembly directly in code.
    ///
    /// ---
    ///
    /// 从字符串汇编并加载汇编源代码。
    ///
    /// 适用于测试或在代码中直接嵌入汇编。
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

    /// Checks if a file is an assembly file based on extension.
    ///
    /// Returns `true` for .s, .S, .asm, .ASM extensions.
    ///
    /// ---
    ///
    /// 根据扩展名检查文件是否为汇编文件。
    ///
    /// 对于 .s, .S, .asm, .ASM 扩展名返回 `true`。
    pub fn is_assembly_file(file_name: &str) -> bool {
        let path = Path::new(file_name);
        if let Some(ext) = path.extension() {
            matches!(ext.to_str(), Some("s") | Some("S") | Some("asm") | Some("ASM"))
        } else {
            false
        }
    }

    /// Runs the virtual machine until halt or exception.
    ///
    /// Executes instructions in a loop until the CPU halts
    /// or encounters an exception.
    ///
    /// ---
    ///
    /// 运行虚拟机直到停止或异常。
    ///
    /// 循环执行指令直到 CPU 停止或遇到异常。
    pub fn run(&mut self) {
        if !self.initialized {
            eprintln!("VM not initialized");
            return;
        }

        if let Some(ref mut cpu) = self.cpu {
            cpu.run();
        }
    }

    /// Executes a single instruction.
    ///
    /// Performs one fetch-decode-execute cycle.
    /// Prints PC after step if tracing is enabled.
    ///
    /// ---
    ///
    /// 执行单条指令。
    ///
    /// 执行一次取指-解码-执行周期。
    /// 如果启用追踪，执行后打印 PC。
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

    /// Halts the virtual machine.
    ///
    /// Stops execution immediately.
    ///
    /// ---
    ///
    /// 停止虚拟机。
    ///
    /// 立即停止执行。
    pub fn halt(&mut self) {
        if self.initialized {
            if let Some(ref mut cpu) = self.cpu {
                cpu.halt();
            }
        }
    }

    /// Checks if the VM is currently running.
    ///
    /// ---
    ///
    /// 检查虚拟机是否正在运行。
    pub fn is_running(&self) -> bool {
        self.initialized && self.cpu.as_ref().unwrap().is_running()
    }

    /// Starts the CPU execution.
    ///
    /// Sets the CPU state to Running, allowing step() to execute.
    ///
    /// ---
    ///
    /// 启动 CPU 执行。
    ///
    /// 将 CPU 状态设置为 Running，允许 step() 执行。
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

    // ========================================
    // Debugger Methods / 调试器方法
    // ========================================

    /// Returns a reference to the debugger, if enabled.
    ///
    /// ---
    ///
    /// 返回调试器引用（如已启用）。
    pub fn get_debugger(&self) -> Option<&debugger::InteractiveDebugger> {
        self.debugger.as_ref()
    }

    /// Returns a mutable reference to the debugger, if enabled.
    ///
    /// ---
    ///
    /// 返回调试器的可变引用（如已启用）。
    pub fn get_debugger_mut(&mut self) -> Option<&mut debugger::InteractiveDebugger> {
        self.debugger.as_mut()
    }

    /// Checks if debugger is enabled.
    ///
    /// ---
    ///
    /// 检查调试器是否启用。
    pub fn is_debugger_enabled(&self) -> bool {
        self.debugger.is_some()
    }

    /// Runs the interactive debugger loop.
    ///
    /// This is the main entry point for the debugger.
    /// Executes until the user quits or the program ends.
    ///
    /// ---
    ///
    /// 运行交互式调试器循环。
    ///
    /// 这是调试器的主入口点。
    /// 执行直到用户退出或程序结束。
    pub fn run_debugger(&mut self) {
        if !self.initialized {
            eprintln!("VM not initialized");
            return;
        }

        if self.debugger.is_none() {
            eprintln!("Debugger not enabled. Use VMConfig::with_debugger() to enable.");
            return;
        }

        println!("RISC-V 128-bit VM Interactive Debugger");
        println!("Type 'help' or '?' for commands.");
        println!();

        // Show initial state
        self.show_debug_state();

        // Start debugger loop
        self.debugger.as_mut().unwrap().action = debugger::DebuggerAction::Stop;

        let stdin = io::stdin();
        loop {
            // Print prompt
            print!("(riscv) ");
            io::stdout().flush().unwrap();

            // Read command
            let mut input = String::new();
            if stdin.lock().read_line(&mut input).is_err() {
                break;
            }

            let input = input.trim();
            if input.is_empty() {
                continue;
            }

            // Parse and execute command
            let cmd = debugger::CommandParser::parse(input);
            match cmd {
                Some(debugger::DebugCommand::Quit) => {
                    println!("Exiting debugger.");
                    break;
                }
                Some(debugger::DebugCommand::Help) => {
                    self.debugger.as_mut().unwrap().print_help();
                }
                Some(cmd) => {
                    if !self.execute_debug_command(cmd) {
                        break;
                    }
                }
                None => {
                    println!("Unknown command. Type 'help' for help.");
                }
            }
        }
    }

    /// Executes a single debugger command.
    ///
    /// Returns false if the debugger should exit.
    ///
    /// ---
    ///
    /// 执行单个调试器命令。
    ///
    /// 如果调试器应退出则返回 false。
    fn execute_debug_command(&mut self, cmd: debugger::DebugCommand) -> bool {
        match cmd {
            debugger::DebugCommand::Quit => {
                println!("Exiting debugger.");
                return false;
            }
            debugger::DebugCommand::Help => {
                self.debugger.as_mut().unwrap().print_help();
            }
            debugger::DebugCommand::Continue => {
                self.debug_continue();
            }
            debugger::DebugCommand::Step => {
                self.debug_step();
            }
            debugger::DebugCommand::Next => {
                self.debug_next();
            }
            debugger::DebugCommand::Finish => {
                self.debug_finish();
            }
            debugger::DebugCommand::Until(addr) => {
                self.debug_until(addr);
            }
            debugger::DebugCommand::Break(addr) => {
                let id = self.debugger.as_mut().unwrap().breakpoints.add_address(addr);
                println!("Breakpoint {} set at 0x{:016x}", id, addr);
            }
            debugger::DebugCommand::TBreak(addr) => {
                let id = self.debugger.as_mut().unwrap().breakpoints.add_temporary(addr);
                println!("Temporary breakpoint {} set at 0x{:016x}", id, addr);
            }
            debugger::DebugCommand::Delete(Some(id)) => {
                if self.debugger.as_mut().unwrap().breakpoints.remove(id).is_some() {
                    println!("Breakpoint {} deleted.", id);
                } else {
                    println!("No breakpoint {} found.", id);
                }
            }
            debugger::DebugCommand::Delete(None) => {
                self.debugger.as_mut().unwrap().breakpoints.clear();
                println!("All breakpoints deleted.");
            }
            debugger::DebugCommand::Enable(id) => {
                if self.debugger.as_mut().unwrap().breakpoints.enable(id) {
                    println!("Breakpoint {} enabled.", id);
                } else {
                    println!("No breakpoint {} found.", id);
                }
            }
            debugger::DebugCommand::Disable(id) => {
                if self.debugger.as_mut().unwrap().breakpoints.disable(id) {
                    println!("Breakpoint {} disabled.", id);
                } else {
                    println!("No breakpoint {} found.", id);
                }
            }
            debugger::DebugCommand::Ignore(id, count) => {
                if let Some(bp) = self.debugger.as_mut().unwrap().breakpoints.get_mut(&id) {
                    bp.ignore_count = count;
                    println!("Breakpoint {} will be ignored for {} hits.", id, count);
                } else {
                    println!("No breakpoint {} found.", id);
                }
            }
            debugger::DebugCommand::ListBreakpoints => {
                self.debugger.as_ref().unwrap().show_breakpoints();
            }
            debugger::DebugCommand::Watch(addr, size, typ) => {
                let id = self.debugger.as_mut().unwrap().watchpoints.add_memory(addr, size, typ);
                println!("Watchpoint {} set at 0x{:016x} ({} bytes)", id, addr, size);
            }
            debugger::DebugCommand::WatchReg(reg, is_fp) => {
                let id = self.debugger.as_mut().unwrap().watchpoints.add_register(reg, is_fp);
                let reg_type = if is_fp { "fp" } else { "gp" };
                println!("Register watchpoint {} set on {}r{}", id, reg_type, reg);
            }
            debugger::DebugCommand::DeleteWatch(id) => {
                if self.debugger.as_mut().unwrap().watchpoints.remove(id) {
                    println!("Watchpoint {} deleted.", id);
                } else {
                    println!("No watchpoint {} found.", id);
                }
            }
            debugger::DebugCommand::ListWatchpoints => {
                self.debugger.as_ref().unwrap().show_watchpoints();
            }
            debugger::DebugCommand::PrintRegister(reg) => {
                if let Some(cpu) = &self.cpu {
                    let regs = cpu.get_registers();
                    let reg_index = unsafe { std::mem::transmute::<u8, register::RegisterIndex>(reg) };
                    let value = regs.read(reg_index);
                    self.debugger.as_ref().unwrap().show_register(reg, value);
                }
            }
            debugger::DebugCommand::PrintAllRegisters => {
                if let Some(cpu) = &self.cpu {
                    let regs = cpu.get_registers();
                    let mut reg_values = [0u128; 32];
                    for i in 0..32 {
                        let reg_index = unsafe { std::mem::transmute::<u8, register::RegisterIndex>(i as u8) };
                        reg_values[i] = regs.read(reg_index);
                    }
                    self.debugger.as_ref().unwrap().show_all_registers(&reg_values, regs.get_pc());
                }
            }
            debugger::DebugCommand::PrintMemory(addr, size) => {
                let mem = self.memory.borrow();
                self.debugger.as_ref().unwrap().show_memory(&mem, addr, size);
            }
            debugger::DebugCommand::SetRegister(reg, value) => {
                if let Some(cpu) = &mut self.cpu {
                    let reg_index = unsafe { std::mem::transmute::<u8, register::RegisterIndex>(reg) };
                    cpu.get_registers_mut().write(reg_index, value);
                    println!("x{} set to 0x{:032x}", reg, value);
                }
            }
            debugger::DebugCommand::SetMemory(addr, data) => {
                self.memory.borrow_mut().write_bytes(addr, &data, data.len());
                println!("Memory at 0x{:016x} updated.", addr);
            }
            debugger::DebugCommand::Disassemble(addr, count) => {
                let mem = self.memory.borrow();
                self.debugger.as_ref().unwrap().show_disassembly(&mem, addr, count);
            }
            debugger::DebugCommand::History(count) => {
                self.debugger.as_ref().unwrap().show_history(count);
            }
            debugger::DebugCommand::Backtrace => {
                println!("Backtrace not yet implemented.");
            }
            debugger::DebugCommand::Where => {
                self.show_debug_state();
            }
            debugger::DebugCommand::Reset => {
                self.reset();
                println!("VM reset.");
                self.show_debug_state();
            }
            debugger::DebugCommand::BreakConditional(addr, condition) => {
                let id = self.debugger.as_mut().unwrap().breakpoints.add_conditional(addr, condition);
                println!("Conditional breakpoint {} set at 0x{:016x}", id, addr);
            }
            debugger::DebugCommand::Help => {
                self.debugger.as_ref().unwrap().print_help();
            }
            debugger::DebugCommand::Raw(cmd) => {
                println!("Unknown command: {}. Type 'help' for available commands.", cmd);
            }
            _ => {
                println!("Command not yet implemented.");
            }
        }
        true
    }

    /// Shows current debug state.
    ///
    /// ---
    ///
    /// 显示当前调试状态。
    fn show_debug_state(&self) {
        if let Some(cpu) = &self.cpu {
            let pc = cpu.get_registers().get_pc();
            let instr = cpu.fetch_instruction(pc);
            let cycle = cpu.get_cycle_count();
            self.debugger.as_ref().unwrap().show_position(pc, instr, cycle);
        }
    }

    /// Continues execution until breakpoint or halt.
    ///
    /// ---
    ///
    /// 继续执行直到断点或停止。
    fn debug_continue(&mut self) {
        if let Some(ref mut cpu) = self.cpu {
            cpu.start();
        }
        self.debugger.as_mut().unwrap().action = debugger::DebuggerAction::Continue;

        loop {
            if !self.is_running() {
                println!("Program stopped.");
                break;
            }

            // Record history before execution
            let history_entry = self.record_history_entry();

            // Execute one step
            if let Some(ref mut cpu) = self.cpu {
                cpu.step();
            }

            // Add to history
            if let Some(entry) = history_entry {
                self.debugger.as_mut().unwrap().history.add(entry);
            }

            // Check breakpoints
            let pc = self.cpu.as_ref().map(|c| c.get_registers().get_pc()).unwrap_or(0);
            
            // First check if breakpoint exists and get its info
            let bp_info = self.debugger.as_mut().unwrap().breakpoints.get_at(pc).map(|bp| {
                (bp.id, bp.enabled, bp.hit_count, bp.ignore_count, bp.typ.clone())
            });
            
            if let Some((id, enabled, mut hit_count, ignore_count, typ)) = bp_info {
                hit_count += 1;
                
                // Update hit count
                if let Some(bp) = self.debugger.as_mut().unwrap().breakpoints.get_mut(&id) {
                    bp.hit_count = hit_count;
                }
                
                if enabled && hit_count > ignore_count {
                    println!("Breakpoint {} hit at 0x{:016x}", id, pc);
                    self.show_debug_state();

                    // Remove temporary breakpoint
                    if matches!(typ, debugger::BreakpointType::Temporary(_)) {
                        self.debugger.as_mut().unwrap().breakpoints.remove(id);
                    }
                    break;
                }
            }
        }
    }

    /// Executes a single step.
    ///
    /// ---
    ///
    /// 执行单步。
    fn debug_step(&mut self) {
        if !self.is_running() {
            self.start();
        }

        // Record history
        let history_entry = self.record_history_entry();

        // Execute step
        if let Some(ref mut cpu) = self.cpu {
            cpu.step();
        }

        // Add to history
        if let Some(entry) = history_entry {
            self.debugger.as_mut().unwrap().history.add(entry);
        }

        self.show_debug_state();
    }

    /// Steps over (doesn't enter function calls).
    ///
    /// ---
    ///
    /// 步过（不进入函数调用）。
    fn debug_next(&mut self) {
        if !self.is_running() {
            self.start();
        }

        // Get current instruction to check if it's a call
        let pc = self.cpu.as_ref().map(|c| c.get_registers().get_pc()).unwrap_or(0);
        let instr = self.memory.borrow().read_32(pc);

        // Simple heuristic: if instruction is JAL or JALR, set temporary breakpoint at next instruction
        let opcode = instr & 0x7f;
        let is_call = opcode == 0x6f || opcode == 0x67; // JAL or JALR

        if is_call {
            // Set temporary breakpoint at next instruction
            let next_pc = pc + 4;
            let id = self.debugger.as_mut().unwrap().breakpoints.add_temporary(next_pc);
            self.debugger.as_mut().unwrap().temp_breakpoint = Some(id);
            self.debug_continue();
        } else {
            self.debug_step();
        }
    }

    /// Steps out of current function.
    ///
    /// ---
    ///
    /// 步出当前函数。
    fn debug_finish(&mut self) {
        if !self.is_running() {
            self.start();
        }

        // Get return address from ra (x1)
        let return_addr = if let Some(cpu) = &self.cpu {
            let regs = cpu.get_registers();
            let ra_index = unsafe { std::mem::transmute::<u8, register::RegisterIndex>(1) };
            regs.read(ra_index)
        } else {
            0
        };

        if return_addr == 0 {
            println!("No return address found.");
            return;
        }

        // Set temporary breakpoint at return address
        let id = self.debugger.as_mut().unwrap().breakpoints.add_temporary(return_addr);
        self.debugger.as_mut().unwrap().temp_breakpoint = Some(id);
        println!("Running until return address 0x{:016x}", return_addr);
        self.debug_continue();
    }

    /// Runs until a specific address.
    ///
    /// ---
    ///
    /// 运行到指定地址。
    fn debug_until(&mut self, addr: memory::Address128) {
        if !self.is_running() {
            self.start();
        }

        let id = self.debugger.as_mut().unwrap().breakpoints.add_temporary(addr);
        self.debugger.as_mut().unwrap().temp_breakpoint = Some(id);
        println!("Running until 0x{:016x}", addr);
        self.debug_continue();
    }

    /// Records a history entry before execution.
    ///
    /// ---
    ///
    /// 执行前记录历史条目。
    fn record_history_entry(&self) -> Option<debugger::HistoryEntry> {
        if let Some(cpu) = &self.cpu {
            let regs = cpu.get_registers();
            let pc = regs.get_pc();
            let instr = cpu.fetch_instruction(pc);
            let cycle = cpu.get_cycle_count();

            let mut reg_values = [0u128; 32];
            for i in 0..32 {
                let reg_index = unsafe { std::mem::transmute::<u8, register::RegisterIndex>(i as u8) };
                reg_values[i] = regs.read(reg_index);
            }

            Some(debugger::HistoryEntry::new(cycle, pc, instr, reg_values, pc))
        } else {
            None
        }
    }
}