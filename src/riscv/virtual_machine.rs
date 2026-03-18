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
use crate::riscv::memory;
use crate::riscv::register;
use std::cell::RefCell;
use std::fs;
use std::io;
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
#[derive(Clone, Copy)]
pub struct VMConfig {
    /// Memory size in bytes (内存大小，单位字节)
    pub(crate) memory_size: memory::Address128,
    /// Enable debug output (启用调试输出)
    pub(crate) debug_mode: bool,
    /// Enable execution tracing (启用执行追踪)
    pub(crate) trace_enabled: bool,
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
            max_cycles: 0,
        }
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
        VirtualMachine {
            memory: Rc::new(RefCell::new(memory::Memory::new(config.memory_size))),
            cpu: None,
            config: config.clone(),
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
}