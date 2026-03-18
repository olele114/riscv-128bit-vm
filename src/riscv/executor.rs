//! Instruction Executor Module
//!
//! Executes decoded RISC-V instructions on the CPU.
//! Implements all instruction types: R-type, I-type, S-type,
//! B-type, U-type, and J-type.
//!
//! # Instruction Categories
//!
//! - **R-type**: ADD, SUB, SLL, SLT, SLTU, XOR, SRL, SRA, OR, AND
//! - **I-type**: ADDI, SLTI, SLTIU, XORI, ORI, ANDI, SLLI, SRLI, SRAI
//! - **Load**: LB, LH, LW, LD, LQ, LBU, LHU, LWU, LDU
//! - **Store**: SB, SH, SW, SD, SQ
//! - **Branch**: BEQ, BNE, BLT, BGE, BLTU, BGEU
//! - **U-type**: LUI, AUIPC
//! - **J-type**: JAL, JALR
//! - **System**: ECALL, EBREAK
//!
//! ---
//!
//! 指令执行器模块
//!
//! 在 CPU 上执行解码后的 RISC-V 指令。
//! 实现所有指令类型：R-type、I-type、S-type、B-type、U-type 和 J-type。
//!
//! # 指令类别
//!
//! - **R-type**: ADD, SUB, SLL, SLT, SLTU, XOR, SRL, SRA, OR, AND
//! - **I-type**: ADDI, SLTI, SLTIU, XORI, ORI, ANDI, SLLI, SRLI, SRAI
//! - **Load**: LB, LH, LW, LD, LQ, LBU, LHU, LWU, LDU
//! - **Store**: SB, SH, SW, SD, SQ
//! - **Branch**: BEQ, BNE, BLT, BGE, BLTU, BGEU
//! - **U-type**: LUI, AUIPC
//! - **J-type**: JAL, JALR
//! - **System**: ECALL, EBREAK

#![allow(dead_code)]

use crate::riscv::instruction;
use crate::riscv::memory;
use crate::riscv::register;
use crate::riscv::cpu;

/// Result of instruction execution.
///
/// Contains information about whether the instruction succeeded,
/// whether a branch was taken, and the next PC if applicable.
///
/// ---
///
/// 指令执行结果。
///
/// 包含指令是否成功、是否发生分支以及下一个 PC（如适用）的信息。
pub struct ExecutionResult {
    /// Whether execution succeeded (执行是否成功)
    pub(crate) success: bool,
    /// Whether a branch was taken (是否发生分支)
    pub(crate) branch_taken: bool,
    /// Next PC if branch was taken (发生分支时的下一个 PC)
    pub(crate) next_pc: memory::Address128,
    /// Error message if execution failed (执行失败时的错误信息)
    error_message: String,
}

/// Instruction Executor.
///
/// Provides static methods for executing all RISC-V instructions.
///
/// ---
///
/// 指令执行器。
///
/// 提供执行所有 RISC-V 指令的静态方法。
pub struct Executor {}

impl ExecutionResult {
    /// Creates a new successful execution result.
    ///
    /// ---
    ///
    /// 创建新的成功执行结果。
    pub fn new() -> Self {
        Self {
            success: true,
            branch_taken: false,
            next_pc: 0,
            error_message: String::new(),
        }
    }
}

impl Executor {
    /// Gets the value of a register.
    ///
    /// Register 0 always returns 0 (hardwired zero).
    ///
    /// ---
    ///
    /// 获取寄存器的值。
    ///
    /// 寄存器 0 始终返回 0（硬连线零）。
    fn get_reg_value(regs: &register::Register, reg: u8) -> i128 {
        if reg == 0 {
            return 0;
        }
        if reg > 31 {
            panic!("Invalid register index: {}", reg);
        }
        regs.read(unsafe { std::mem::transmute::<u8, register::RegisterIndex>(reg) }) as i128
    }

    /// Sets the value of a register.
    ///
    /// Writes to register 0 are ignored (hardwired zero).
    ///
    /// ---
    ///
    /// 设置寄存器的值。
    ///
    /// 对寄存器 0 的写入被忽略（硬连线零）。
    fn set_reg_value(regs: &mut register::Register, reg: u8, value: i128) {
        if reg == 0 {
            return;
        }
        if reg > 31 {
            panic!("Invalid register index: {}", reg);
        }
        regs.write(unsafe { std::mem::transmute::<u8, register::RegisterIndex>(reg) }, value as u128);
    }

    /// Executes a decoded instruction on the CPU.
    ///
    /// Dispatches to the appropriate instruction handler based on opcode.
    ///
    /// ---
    ///
    /// 在 CPU 上执行解码后的指令。
    ///
    /// 根据操作码分派到相应的指令处理程序。
    pub fn execute(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let mut result = ExecutionResult::new();

        match decoded.opcode {
            instruction::OpCode::Lui => Self::execute_lui(cpu, decoded),
            instruction::OpCode::Auipc => Self::execute_auipc(cpu, decoded),
            instruction::OpCode::Jal => Self::execute_jal(cpu, decoded),
            instruction::OpCode::Jalr => Self::execute_jalr(cpu, decoded),

            instruction::OpCode::Branch => {
                match decoded.funct3 {
                    0x0 => Self::execute_beq(cpu, decoded),
                    0x1 => Self::execute_bne(cpu, decoded),
                    0x4 => Self::execute_blt(cpu, decoded),
                    0x5 => Self::execute_bge(cpu, decoded),
                    0x6 => Self::execute_bltu(cpu, decoded),
                    0x7 => Self::execute_bgeu(cpu, decoded),
                    _ => {
                        result.success = false;
                        result.error_message = "Unknown branch func3".to_string();
                        result
                    }
                }
            }

            instruction::OpCode::Load => {
                match decoded.funct3 {
                    0x0 => Self::execute_lb(cpu, decoded),
                    0x1 => Self::execute_lh(cpu, decoded),
                    0x2 => Self::execute_lw(cpu, decoded),
                    0x3 => Self::execute_ld(cpu, decoded),
                    0x4 => Self::execute_lq(cpu, decoded),
                    0x5 => Self::execute_lhu(cpu, decoded),
                    0x6 => Self::execute_lwu(cpu, decoded),
                    0x7 => Self::execute_ldu(cpu, decoded),
                    _ => {
                        result.success = false;
                        result.error_message = "Unknown load func3".to_string();
                        result
                    }
                }
            }

            instruction::OpCode::Store => {
                match decoded.funct3 {
                    0x0 => Self::execute_sb(cpu, decoded),
                    0x1 => Self::execute_sh(cpu, decoded),
                    0x2 => Self::execute_sw(cpu, decoded),
                    0x3 => Self::execute_sd(cpu, decoded),
                    0x4 => Self::execute_sq(cpu, decoded),
                    _ => {
                        result.success = false;
                        result.error_message = "Unknown store func3".to_string();
                        result
                    }
                }
            }

            instruction::OpCode::Imm => {
                match decoded.funct3 {
                    0x0 => Self::execute_addi(cpu, decoded),
                    0x1 => Self::execute_slli(cpu, decoded),
                    0x2 => Self::execute_slti(cpu, decoded),
                    0x3 => Self::execute_sltiu(cpu, decoded),
                    0x4 => Self::execute_xori(cpu, decoded),
                    0x5 => if (decoded.funct7 & 0x20) != 0 { Self::execute_srai(cpu, decoded) } else { Self::execute_srli(cpu, decoded) },
                    0x6 => Self::execute_ori(cpu, decoded),
                    0x7 => Self::execute_andi(cpu, decoded),
                    _ => {
                        result.success = false;
                        result.error_message = "Unknown imm func3".to_string();
                        result
                    }
                }
            }

            instruction::OpCode::Reg => {
                 match decoded.funct3 {
                     0x0 => if (decoded.funct7 & 0x20) != 0 { Self::execute_sub(cpu, decoded) } else { Self::execute_add(cpu, decoded) },
                     0x1 => Self::execute_sll(cpu, decoded),
                     0x2 => Self::execute_slt(cpu, decoded),
                     0x3 => Self::execute_sltu(cpu, decoded),
                     0x4 => Self::execute_xor(cpu, decoded),
                     0x5 => if (decoded.funct7 & 0x20) != 0 { Self::execute_sra(cpu, decoded) } else { Self::execute_srl(cpu, decoded) },
                     0x6 => Self::execute_or(cpu, decoded),
                     0x7 => Self::execute_and(cpu, decoded),
                     _ => {
                         result.success = false;
                         result.error_message = "Unknown reg func3".to_string();
                         result
                     }
                 }
            }

            instruction::OpCode::System => {
                if decoded.funct3 == 0x0 {
                    if decoded.imm == 0 { return Self::execute_ecall(cpu) }
                    if decoded.imm == 1 { return Self::execute_ebreak(cpu) }
                }
                result.success = false;
                result.error_message = "Unknown system instruction".to_string();
                result
            }

            _ => {
                result.success = false;
                result.error_message = "Unknown opcode".to_string();
                result
            }
        }
    }

    // ========================================
    // R-type Instructions / R 型指令
    // ========================================

    /// ADD: Add registers (寄存器加法)
    ///
    /// rd = rs1 + rs2
    fn execute_add(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers_mut();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1);
        let rs2_val = Self::get_reg_value(regs, decoded.rs2);
        let rd_val = rs1_val + rs2_val;
        Self::set_reg_value(regs, decoded.rd, rd_val);
        result
    }

    /// SUB: Subtract registers (寄存器减法)
    ///
    /// rd = rs1 - rs2
    fn execute_sub(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers_mut();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1);
        let rs2_val = Self::get_reg_value(regs, decoded.rs2);
        let rd_val = rs1_val - rs2_val;
        Self::set_reg_value(regs, decoded.rd, rd_val);
        result
    }

    /// SLL: Shift left logical (逻辑左移)
    ///
    /// rd = rs1 << (rs2 & 0x7f)
    fn execute_sll(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers_mut();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1);
        let rs2_val = Self::get_reg_value(regs, decoded.rs2);
        let shift = (rs2_val & 0x7f) as u128;
        let rd_val = rs1_val << shift;
        Self::set_reg_value(regs, decoded.rd, rd_val);
        result
    }

    /// SLT: Set less than (有符号小于设置)
    ///
    /// rd = (rs1 < rs2) ? 1 : 0 (signed comparison)
    fn execute_slt(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers_mut();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1);
        let rs2_val = Self::get_reg_value(regs, decoded.rs2);
        let rd_val = if rs1_val < rs2_val { 1 } else { 0 };
        Self::set_reg_value(regs, decoded.rd, rd_val);
        result
    }

    /// SLTU: Set less than unsigned (无符号小于设置)
    ///
    /// rd = (rs1 < rs2) ? 1 : 0 (unsigned comparison)
    fn execute_sltu(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers_mut();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1) as u128;
        let rs2_val = Self::get_reg_value(regs, decoded.rs2) as u128;
        let rd_val = if rs1_val < rs2_val { 1 } else { 0 };
        Self::set_reg_value(regs, decoded.rd, rd_val);
        result
    }

    /// XOR: Bitwise exclusive OR (按位异或)
    ///
    /// rd = rs1 ^ rs2
    fn execute_xor(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers_mut();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1);
        let rs2_val = Self::get_reg_value(regs, decoded.rs2);
        let rd_val = rs1_val ^ rs2_val;
        Self::set_reg_value(regs, decoded.rd, rd_val);
        result
    }

    /// SRL: Shift right logical (逻辑右移)
    ///
    /// rd = rs1 >> (rs2 & 0x7f) (zero-extended)
    fn execute_srl(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers_mut();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1) as u128;
        let rs2_val = Self::get_reg_value(regs, decoded.rs2);
        let shift = (rs2_val & 0x7f) as u128;
        let rd_val = (rs1_val >> shift) as i128;
        Self::set_reg_value(regs, decoded.rd, rd_val);
        result
    }

    /// SRA: Shift right arithmetic (算术右移)
    ///
    /// rd = rs1 >> (rs2 & 0x7f) (sign-extended)
    fn execute_sra(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers_mut();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1);
        let rs2_val = Self::get_reg_value(regs, decoded.rs2);
        let shift = (rs2_val & 0x7f) as u128;
        let rd_val = rs1_val >> shift;
        Self::set_reg_value(regs, decoded.rd, rd_val);
        result
    }

    /// OR: Bitwise OR (按位或)
    ///
    /// rd = rs1 | rs2
    fn execute_or(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers_mut();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1);
        let rs2_val = Self::get_reg_value(regs, decoded.rs2);
        let rd_val = rs1_val | rs2_val;
        Self::set_reg_value(regs, decoded.rd, rd_val);
        result
    }

    /// AND: Bitwise AND (按位与)
    ///
    /// rd = rs1 & rs2
    fn execute_and(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers_mut();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1);
        let rs2_val = Self::get_reg_value(regs, decoded.rs2);
        let rd_val = rs1_val & rs2_val;
        Self::set_reg_value(regs, decoded.rd, rd_val);
        result
    }

    // ========================================
    // I-type Instructions / I 型指令
    // ========================================

    /// ADDI: Add immediate (立即数加法)
    ///
    /// rd = rs1 + imm
    fn execute_addi(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers_mut();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1);
        let rd_val = rs1_val + decoded.imm;
        Self::set_reg_value(regs, decoded.rd, rd_val);
        result
    }

    /// SLLI: Shift left logical immediate (立即数逻辑左移)
    ///
    /// rd = rs1 << (imm & 0x7f)
    fn execute_slli(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers_mut();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1);
        let shift = (decoded.imm & 0x7f) as u128;
        let rd_val = rs1_val << shift;
        Self::set_reg_value(regs, decoded.rd, rd_val);
        result
    }

    /// SLTI: Set less than immediate (有符号立即数小于设置)
    ///
    /// rd = (rs1 < imm) ? 1 : 0 (signed comparison)
    fn execute_slti(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers_mut();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1);
        let imm_val = decoded.imm;
        let rd_val = if rs1_val < imm_val { 1 } else { 0 };
        Self::set_reg_value(regs, decoded.rd, rd_val);
        result
    }

    /// SLTIU: Set less than immediate unsigned (无符号立即数小于设置)
    ///
    /// rd = (rs1 < imm) ? 1 : 0 (unsigned comparison)
    fn execute_sltiu(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers_mut();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1) as u128;
        let imm_val = decoded.imm as u128;
        let rd_val = if rs1_val < imm_val { 1 } else { 0 };
        Self::set_reg_value(regs, decoded.rd, rd_val);
        result
    }

    /// XORI: XOR immediate (立即数异或)
    ///
    /// rd = rs1 ^ imm
    fn execute_xori(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers_mut();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1);
        let rd_val = rs1_val ^ decoded.imm;
        Self::set_reg_value(regs, decoded.rd, rd_val);
        result
    }

    /// SRLI: Shift right logical immediate (立即数逻辑右移)
    ///
    /// rd = rs1 >> (imm & 0x7f) (zero-extended)
    fn execute_srli(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers_mut();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1) as u128;
        let shift = (decoded.imm & 0x7f) as u128;
        let rd_val = (rs1_val >> shift) as i128;
        Self::set_reg_value(regs, decoded.rd, rd_val);
        result
    }

    /// SRAI: Shift right arithmetic immediate (立即数算术右移)
    ///
    /// rd = rs1 >> (imm & 0x7f) (sign-extended)
    fn execute_srai(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers_mut();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1);
        let shift = (decoded.imm & 0x7f) as u128;
        let rd_val = rs1_val >> shift;
        Self::set_reg_value(regs, decoded.rd, rd_val);
        result
    }

    /// ORI: OR immediate (立即数或)
    ///
    /// rd = rs1 | imm
    fn execute_ori(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers_mut();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1);
        let rd_val = rs1_val | decoded.imm;
        Self::set_reg_value(regs, decoded.rd, rd_val);
        result
    }

    /// ANDI: AND immediate (立即数与)
    ///
    /// rd = rs1 & imm
    fn execute_andi(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers_mut();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1);
        let rd_val = rs1_val & decoded.imm;
        Self::set_reg_value(regs, decoded.rd, rd_val);
        result
    }

    // ========================================
    // Load Instructions / 加载指令
    // ========================================

    /// LB: Load byte (加载字节，符号扩展)
    fn execute_lb(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let rs1_val = Self::get_reg_value(cpu.get_registers(), decoded.rs1);
        let addr = (rs1_val + decoded.imm) as memory::Address128;
        let rd_val = cpu.get_memory().borrow().read_8(addr);
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, rd_val as i128);
        result
    }

    /// LH: Load halfword (加载半字，符号扩展)
    fn execute_lh(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let rs1_val = Self::get_reg_value(cpu.get_registers(), decoded.rs1);
        let addr = (rs1_val + decoded.imm) as memory::Address128;
        let rd_val = cpu.get_memory().borrow().read_16(addr);
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, rd_val as i128);
        result
    }

    /// LW: Load word (加载字，符号扩展)
    fn execute_lw(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let rs1_val = Self::get_reg_value(cpu.get_registers(), decoded.rs1);
        let addr = (rs1_val + decoded.imm) as memory::Address128;
        let rd_val = cpu.get_memory().borrow().read_32(addr);
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, rd_val as i128);
        result
    }

    /// LD: Load doubleword (加载双字，符号扩展)
    fn execute_ld(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let rs1_val = Self::get_reg_value(cpu.get_registers(), decoded.rs1);
        let addr = (rs1_val + decoded.imm) as memory::Address128;
        let rd_val = cpu.get_memory().borrow().read_64(addr);
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, rd_val as i128);
        result
    }

    /// LQ: Load quadword (加载四字，符号扩展)
    fn execute_lq(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let rs1_val = Self::get_reg_value(cpu.get_registers(), decoded.rs1);
        let addr = (rs1_val + decoded.imm) as memory::Address128;
        let rd_val = cpu.get_memory().borrow().read_128(addr);
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, rd_val as i128);
        result
    }

    /// LBU: Load byte unsigned (加载字节，零扩展)
    fn execute_lbu(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let rs1_val = Self::get_reg_value(cpu.get_registers(), decoded.rs1);
        let addr = (rs1_val + decoded.imm) as memory::Address128;
        let rd_val = cpu.get_memory().borrow().read_8(addr) as u128;
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, rd_val as i128);
        result
    }

    /// LHU: Load halfword unsigned (加载半字，零扩展)
    fn execute_lhu(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let rs1_val = Self::get_reg_value(cpu.get_registers(), decoded.rs1);
        let addr = (rs1_val + decoded.imm) as memory::Address128;
        let rd_val = cpu.get_memory().borrow().read_16(addr) as u128;
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, rd_val as i128);
        result
    }

    /// LWU: Load word unsigned (加载字，零扩展)
    fn execute_lwu(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let rs1_val = Self::get_reg_value(cpu.get_registers(), decoded.rs1);
        let addr = (rs1_val + decoded.imm) as memory::Address128;
        let rd_val = cpu.get_memory().borrow().read_32(addr) as u128;
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, rd_val as i128);
        result
    }

    /// LDU: Load doubleword unsigned (加载双字，零扩展)
    fn execute_ldu(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let rs1_val = Self::get_reg_value(cpu.get_registers(), decoded.rs1);
        let addr = (rs1_val + decoded.imm) as memory::Address128;
        let rd_val = cpu.get_memory().borrow().read_64(addr) as u128;
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, rd_val as i128);
        result
    }

    // ========================================
    // Store Instructions / 存储指令
    // ========================================

    /// SB: Store byte (存储字节)
    fn execute_sb(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let rs1_val = Self::get_reg_value(cpu.get_registers(), decoded.rs1);
        let rs2_val = Self::get_reg_value(cpu.get_registers(), decoded.rs2);
        let addr = (rs1_val + decoded.imm) as memory::Address128;
        cpu.get_memory().borrow_mut().write_8(addr, rs2_val as u8);
        result
    }

    /// SH: Store halfword (存储半字)
    fn execute_sh(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let rs1_val = Self::get_reg_value(cpu.get_registers(), decoded.rs1);
        let rs2_val = Self::get_reg_value(cpu.get_registers(), decoded.rs2);
        let addr = (rs1_val + decoded.imm) as memory::Address128;
        cpu.get_memory().borrow_mut().write_16(addr, rs2_val as u16);
        result
    }

    /// SW: Store word (存储字)
    fn execute_sw(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let rs1_val = Self::get_reg_value(cpu.get_registers(), decoded.rs1);
        let rs2_val = Self::get_reg_value(cpu.get_registers(), decoded.rs2);
        let addr = (rs1_val + decoded.imm) as memory::Address128;
        cpu.get_memory().borrow_mut().write_32(addr, rs2_val as u32);
        result
    }

    /// SD: Store doubleword (存储双字)
    fn execute_sd(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let rs1_val = Self::get_reg_value(cpu.get_registers(), decoded.rs1);
        let rs2_val = Self::get_reg_value(cpu.get_registers(), decoded.rs2);
        let addr = (rs1_val + decoded.imm) as memory::Address128;
        cpu.get_memory().borrow_mut().write_64(addr, rs2_val as u64);
        result
    }

    /// SQ: Store quadword (存储四字)
    fn execute_sq(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let rs1_val = Self::get_reg_value(cpu.get_registers(), decoded.rs1);
        let rs2_val = Self::get_reg_value(cpu.get_registers(), decoded.rs2);
        let addr = (rs1_val + decoded.imm) as memory::Address128;
        cpu.get_memory().borrow_mut().write_128(addr, rs2_val as memory::Word128);
        result
    }

    // ========================================
    // Branch Instructions / 分支指令
    // ========================================

    /// BEQ: Branch if equal (相等则分支)
    fn execute_beq(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let mut result = ExecutionResult::new();
        let rs1_val = Self::get_reg_value(cpu.get_registers(), decoded.rs1);
        let rs2_val = Self::get_reg_value(cpu.get_registers(), decoded.rs2);

        if rs1_val == rs2_val {
            result.branch_taken = true;
            let pc = cpu.get_registers().get_pc();
            result.next_pc = (pc as i128 + decoded.imm) as memory::Address128;
        }
        result
    }

    /// BNE: Branch if not equal (不等则分支)
    fn execute_bne(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let mut result = ExecutionResult::new();
        let rs1_val = Self::get_reg_value(cpu.get_registers(), decoded.rs1);
        let rs2_val = Self::get_reg_value(cpu.get_registers(), decoded.rs2);

        if rs1_val != rs2_val {
            result.branch_taken = true;
            let pc = cpu.get_registers().get_pc();
            result.next_pc = (pc as i128 + decoded.imm) as memory::Address128;
        }
        result
    }

    /// BLT: Branch if less than (有符号小于则分支)
    fn execute_blt(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let mut result = ExecutionResult::new();
        let rs1_val = Self::get_reg_value(cpu.get_registers(), decoded.rs1);
        let rs2_val = Self::get_reg_value(cpu.get_registers(), decoded.rs2);

        if rs1_val < rs2_val {
            result.branch_taken = true;
            let pc = cpu.get_registers().get_pc();
            result.next_pc = (pc as i128 + decoded.imm) as memory::Address128;
        }
        result
    }

    /// BGE: Branch if greater or equal (有符号大于等于则分支)
    fn execute_bge(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let mut result = ExecutionResult::new();
        let rs1_val = Self::get_reg_value(cpu.get_registers(), decoded.rs1);
        let rs2_val = Self::get_reg_value(cpu.get_registers(), decoded.rs2);

        if rs1_val >= rs2_val {
            result.branch_taken = true;
            let pc = cpu.get_registers().get_pc();
            result.next_pc = (pc as i128 + decoded.imm) as memory::Address128;
        }
        result
    }

    /// BLTU: Branch if less than unsigned (无符号小于则分支)
    fn execute_bltu(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let mut result = ExecutionResult::new();
        let rs1_val = Self::get_reg_value(cpu.get_registers(), decoded.rs1) as u128;
        let rs2_val = Self::get_reg_value(cpu.get_registers(), decoded.rs2) as u128;

        if rs1_val < rs2_val {
            result.branch_taken = true;
            let pc = cpu.get_registers().get_pc();
            result.next_pc = (pc as i128 + decoded.imm) as memory::Address128;
        }
        result
    }

    /// BGEU: Branch if greater or equal unsigned (无符号大于等于则分支)
    fn execute_bgeu(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let mut result = ExecutionResult::new();
        let rs1_val = Self::get_reg_value(cpu.get_registers(), decoded.rs1) as u128;
        let rs2_val = Self::get_reg_value(cpu.get_registers(), decoded.rs2) as u128;

        if rs1_val >= rs2_val {
            result.branch_taken = true;
            let pc = cpu.get_registers().get_pc();
            result.next_pc = (pc as i128 + decoded.imm) as memory::Address128;
        }
        result
    }

    // ========================================
    // U-type Instructions / U 型指令
    // ========================================

    /// LUI: Load upper immediate (加载上位立即数)
    ///
    /// rd = imm (upper 20 bits)
    fn execute_lui(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, decoded.imm);
        result
    }

    /// AUIPC: Add upper immediate to PC (将上位立即数加到 PC)
    ///
    /// rd = PC + imm
    fn execute_auipc(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let pc = cpu.get_registers().get_pc();
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, pc as i128 + decoded.imm);
        result
    }

    // ========================================
    // J-type Instructions / J 型指令
    // ========================================

    /// JAL: Jump and link (跳转并链接)
    ///
    /// rd = PC + 4; PC += imm
    fn execute_jal(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let mut result = ExecutionResult::new();

        let pc = cpu.get_registers().get_pc();
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, pc as i128 + 4);

        result.branch_taken = true;
        result.next_pc = (cpu.get_registers().get_pc() as i128 + decoded.imm) as memory::Address128;
        result
    }

    /// JALR: Jump and link register (寄存器跳转并链接)
    ///
    /// rd = PC + 4; PC = (rs1 + imm) & ~1
    fn execute_jalr(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let mut result = ExecutionResult::new();

        let rs1_val = Self::get_reg_value(cpu.get_registers(), decoded.rs1);
        let pc = cpu.get_registers().get_pc();

        let target_addr = ((rs1_val + decoded.imm) & (!1)) as memory::Address128;

        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, pc as i128 + 4);

        result.branch_taken = true;
        result.next_pc = target_addr;
        result
    }

    // ========================================
    // System Instructions / 系统指令
    // ========================================

    /// ECALL: Environment call (环境调用)
    ///
    /// Raises a system call exception.
    fn execute_ecall(cpu: &mut cpu::CPU) -> ExecutionResult {
        let mut result = ExecutionResult::new();
        cpu.raise_exception(cpu::ExceptionType::SystemCall, 0, cpu.get_registers().get_pc(), String::from("ECALL"));
        result.success = false;
        result.error_message = String::from("System call");
        result
    }

    /// EBREAK: Environment break (环境断点)
    ///
    /// Raises a breakpoint exception.
    fn execute_ebreak(cpu: &mut cpu::CPU) -> ExecutionResult {
        let mut result = ExecutionResult::new();
        cpu.raise_exception(cpu::ExceptionType::Breakpoint, 0, cpu.get_registers().get_pc(), String::from("EBREAK"));
        result.success = false;
        result.error_message = String::from("Breakpoint");
        result
    }
}
