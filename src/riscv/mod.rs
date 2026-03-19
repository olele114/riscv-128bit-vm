//! RISC-V Virtual Machine Module
//!
//! This module contains all components for the RISC-V 128-bit virtual machine:
//!
//! - `memory`: Memory system with configurable size and access methods
//! - `register`: 128-bit register file with standard RISC-V ABI names
//! - `cpu`: CPU core with fetch-decode-execute cycle
//! - `instruction`: Instruction decoder for RISC-V instruction set
//! - `executor`: Instruction execution engine
//! - `virtual_machine`: High-level VM interface
//! - `assembler`: RISC-V assembler for assembly source files
//! - `debugger`: Interactive debugger with breakpoints and watchpoints
//!
//! ---
//!
//! RISC-V 虚拟机模块
//!
//! 此模块包含 RISC-V 128位虚拟机的所有组件：
//!
//! - `memory`: 内存系统，支持可配置大小和访问方法
//! - `register`: 128位寄存器组，使用标准 RISC-V ABI 命名
//! - `cpu`: CPU 核心，实现取指-解码-执行周期
//! - `instruction`: RISC-V 指令集解码器
//! - `executor`: 指令执行引擎
//! - `virtual_machine`: 高级虚拟机接口
//! - `assembler`: RISC-V 汇编器，用于汇编源文件
//! - `debugger`: 交互式调试器，支持断点和观察点

pub mod memory;
pub mod register;
pub mod cpu;
pub mod instruction;
pub mod executor;
pub mod virtual_machine;
pub mod assembler;
pub mod debugger;