//! Register Module
//!
//! Implements the 128-bit register file for RISC-V.
//! Contains 32 general-purpose registers (x0-x31) and special registers (PC, FCSR).
//!
//! # Register ABI Names
//!
//! | Register | ABI Name | Description |
//! |----------|----------|-------------|
//! | x0 | zero | Hardwired zero |
//! | x1 | ra | Return address |
//! | x2 | sp | Stack pointer |
//! | x3 | gp | Global pointer |
//! | x4 | tp | Thread pointer |
//! | x5-x7 | t0-t2 | Temporaries |
//! | x8 | s0/fp | Saved register / Frame pointer |
//! | x9 | s1 | Saved register |
//! | x10-x17 | a0-a7 | Arguments / Return values |
//! | x18-x27 | s2-s11 | Saved registers |
//! | x28-x31 | t3-t6 | Temporaries |
//!
//! ---
//!
//! 寄存器模块
//!
//! 实现 RISC-V 的 128 位寄存器组。
//! 包含 32 个通用寄存器 (x0-x31) 和特殊寄存器 (PC, FCSR)。
//!
//! # 寄存器 ABI 名称
//!
//! | 寄存器 | ABI 名称 | 描述 |
//! |--------|----------|------|
//! | x0 | zero | 硬连线零 |
//! | x1 | ra | 返回地址 |
//! | x2 | sp | 栈指针 |
//! | x3 | gp | 全局指针 |
//! | x4 | tp | 线程指针 |
//! | x5-x7 | t0-t2 | 临时寄存器 |
//! | x8 | s0/fp | 保存寄存器 / 帧指针 |
//! | x9 | s1 | 保存寄存器 |
//! | x10-x17 | a0-a7 | 参数 / 返回值 |
//! | x18-x27 | s2-s11 | 保存寄存器 |
//! | x28-x31 | t3-t6 | 临时寄存器 |

#![allow(dead_code)]

type Register128 = u128;
type Register64 = u64;

/// Register index enumeration.
///
/// Represents the 32 general-purpose registers.
/// x0 is always zero (writes are ignored).
///
/// ---
///
/// 寄存器索引枚举。
///
/// 表示 32 个通用寄存器。
/// x0 始终为零（写入被忽略）。
#[repr(u8)]
#[derive(PartialOrd, PartialEq)]
pub enum RegisterIndex {
    X0 = 0,     // zero
    X1 = 1,     // ra (return address)
    X2 = 2,     // sp (stack pointer)
    X3 = 3,     // gp (global pointer)
    X4 = 4,     // tp (thread pointer)
    X5 = 5,     // t0 (temporary)
    X6 = 6,     // t1
    X7 = 7,     // t2
    X8 = 8,     // s0/fp (saved register/frame pointer)
    X9 = 9,     // s1
    X10 = 10,   // a0 (function argument / return value)
    X11 = 11,   // a1
    X12 = 12,   // a2
    X13 = 13,   // a3
    X14 = 14,   // a4
    X15 = 15,   // a5
    X16 = 16,   // a6
    X17 = 17,   // a7
    X18 = 18,   // s2
    X19 = 19,   // s3
    X20 = 20,   // s4
    X21 = 21,   // s5
    X22 = 22,   // s6
    X23 = 23,   // s7
    X24 = 24,   // s8
    X25 = 25,   // s9
    X26 = 26,   // s10
    X27 = 27,   // s11
    X28 = 28,   // t3
    X29 = 29,   // t4
    X30 = 30,   // t5
    X31 = 31,   // t6
}

/// Special registers (PC, FCSR).
///
/// ---
///
/// 特殊寄存器 (PC, FCSR)。
pub struct SpecialRegisters {
    /// Program counter (程序计数器)
    pc: Register128,
    /// Floating-point control and status register (浮点控制状态寄存器)
    fcsr: Register64,
}

/// Register file with 32 128-bit registers.
///
/// x0 is hardwired to zero - writes are silently ignored.
///
/// ---
///
/// 包含 32 个 128 位寄存器的寄存器组。
///
/// x0 硬连线为零 - 写入被静默忽略。
pub struct Register {
    registers: [Register128; 32],
    special: SpecialRegisters,
}

impl Register {
    /// Creates a new register file with all zeros.
    ///
    /// ---
    ///
    /// 创建所有寄存器为零的新寄存器组。
    pub fn new() -> Self {
        let mut tmp = Self {
            registers: [0; 32],
            special: SpecialRegisters {
                pc: 0,
                fcsr: 0,
            },
        };
        tmp.reset();
        tmp
    }

    /// Reads a register value.
    ///
    /// Reading x0 always returns 0.
    ///
    /// ---
    ///
    /// 读取寄存器值。
    ///
    /// 读取 x0 始终返回 0。
    pub fn read(&self, index: RegisterIndex) -> Register128 {
        if index > RegisterIndex::X31 {
            panic!("Invalid register index: Index out of range");
        }
        self.registers[index as usize]
    }

    /// Writes a value to a register.
    ///
    /// Writes to x0 are ignored (remains zero).
    ///
    /// ---
    ///
    /// 将值写入寄存器。
    ///
    /// 对 x0 的写入被忽略（保持为零）。
    pub fn write(&mut self, index: RegisterIndex, value: Register128) {
        if index > RegisterIndex::X31 {
            panic!("Invalid register index: Index out of range");
        }
        if index != RegisterIndex::X0 {
            self.registers[index as usize] = value;
        }
    }

    /// Returns the program counter.
    ///
    /// ---
    ///
    /// 返回程序计数器。
    pub fn get_pc(&self) -> Register128 {
        self.special.pc
    }

    /// Sets the program counter.
    ///
    /// ---
    ///
    /// 设置程序计数器。
    pub fn set_pc(&mut self, value: Register128) {
        self.special.pc = value;
    }

    /// Resets all registers to zero.
    ///
    /// ---
    ///
    /// 将所有寄存器重置为零。
    pub fn reset(&mut self) {
        self.registers = [0; 32];
        self.special = SpecialRegisters {
            pc: 0,
            fcsr: 0,
        };
    }

    /// Returns a reference to special registers.
    ///
    /// ---
    ///
    /// 返回特殊寄存器的引用。
    pub fn get_special_registers(&self) -> &SpecialRegisters {
        &self.special
    }

    /// Returns a mutable reference to special registers.
    ///
    /// ---
    ///
    /// 返回特殊寄存器的可变引用。
    pub fn get_special_registers_mut(&mut self) -> &mut SpecialRegisters {
        &mut self.special
    }
}