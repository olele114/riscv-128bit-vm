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
//! - **M extension**: MUL, MULH, MULHSU, MULHU, DIV, DIVU, REM, REMU
//! - **A extension**: LR.D, SC.D, AMOADD.D, AMOSWAP.D, AMOAND.D, AMOOR.D, AMOXOR.D, AMOMAX.D, AMOMAXU.D, AMOMIN.D, AMOMINU.D
//! - **F extension (RV32F)**: FLW, FSW, FADD.S, FSUB.S, FMUL.S, FDIV.S, FSQRT.S, FSGNJ.S, FMIN.S, FMAX.S, FCVT.W.S, FCVT.S.W, FMV.X.S, FMV.S.X, FEQ.S, FLT.S, FLE.S, FCLASS.S
//! - **D extension (RV64D)**: FLD, FSD, FADD.D, FSUB.D, FMUL.D, FDIV.D, FSQRT.D, FSGNJ.D, FMIN.D, FMAX.D, FCVT.L.D, FCVT.D.L, FMV.X.D, FMV.D.X, FEQ.D, FLT.D, FLE.D, FCLASS.D
//! - **Q extension (RV128Q)**: FLQ, FSQ, FADD.Q, FSUB.Q, FMUL.Q, FDIV.Q, FSQRT.Q, FSGNJ.Q, FMIN.Q, FMAX.Q, FCVT.Q.L, FCVT.L.Q, FEQ.Q, FLT.Q, FLE.Q, FCLASS.Q
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
//! - **M 扩展**: MUL, MULH, MULHSU, MULHU, DIV, DIVU, REM, REMU
//! - **A 扩展**: LR.D, SC.D, AMOADD.D, AMOSWAP.D, AMOAND.D, AMOOR.D, AMOXOR.D, AMOMAX.D, AMOMAXU.D, AMOMIN.D, AMOMINU.D
//! - **F 扩展 (RV32F)**: FLW, FSW, FADD.S, FSUB.S, FMUL.S, FDIV.S, FSQRT.S, FSGNJ.S, FMIN.S, FMAX.S, FCVT.W.S, FCVT.S.W, FMV.X.S, FMV.S.X, FEQ.S, FLT.S, FLE.S, FCLASS.S
//! - **D 扩展 (RV64D)**: FLD, FSD, FADD.D, FSUB.D, FMUL.D, FDIV.D, FSQRT.D, FSGNJ.D, FMIN.D, FMAX.D, FCVT.L.D, FCVT.D.L, FMV.X.D, FMV.D.X, FEQ.D, FLT.D, FLE.D, FCLASS.D
//! - **Q 扩展 (RV128Q)**: FLQ, FSQ, FADD.Q, FSUB.Q, FMUL.Q, FDIV.Q, FSQRT.Q, FSGNJ.Q, FMIN.Q, FMAX.Q, FCVT.Q.L, FCVT.L.Q, FEQ.Q, FLT.Q, FLE.Q, FCLASS.Q

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
                // Check for Zba/Zbb immediate instructions
                if decoded.funct7 == 0x0C && decoded.funct3 == 0x5 {
                    // Zbb RORI
                    Self::execute_rori(cpu, decoded)
                } else if decoded.funct7 == 0x08 && decoded.funct3 == 0x1 {
                    // Zba SLLI.UW
                    Self::execute_slli_uw(cpu, decoded)
                } else if decoded.funct3 == 0x5 && decoded.funct7 == 0x20 {
                    // Zbb BEXTI
                    Self::execute_bexti(cpu, decoded)
                } else if decoded.funct3 == 0x1 && decoded.funct7 == 0x30 {
                    // Zbb BCLRI
                    Self::execute_bclri(cpu, decoded)
                } else if decoded.funct3 == 0x1 && decoded.funct7 == 0x28 {
                    // Zbb BSETI
                    Self::execute_bseti(cpu, decoded)
                } else if decoded.funct3 == 0x5 && decoded.funct7 == 0x28 {
                    // Zbb BINVI
                    Self::execute_binvi(cpu, decoded)
                } else {
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
            }

            instruction::OpCode::Reg => {
                // Check for M extension instructions (funct7 == 0x01)
                if decoded.funct7 == 0x01 {
                    match decoded.funct3 {
                        0x0 => Self::execute_mul(cpu, decoded),
                        0x1 => Self::execute_mulh(cpu, decoded),
                        0x2 => Self::execute_mulhsu(cpu, decoded),
                        0x3 => Self::execute_mulhu(cpu, decoded),
                        0x4 => Self::execute_div(cpu, decoded),
                        0x5 => Self::execute_divu(cpu, decoded),
                        0x6 => Self::execute_rem(cpu, decoded),
                        0x7 => Self::execute_remu(cpu, decoded),
                        _ => {
                            result.success = false;
                            result.error_message = "Unknown M extension func3".to_string();
                            result
                        }
                    }
                } else if (decoded.funct7 & 0x80) != 0 {
                    // P extension: SIMD instructions have bit 7 set in funct7
                    Self::execute_simd(cpu, decoded)
                } else if decoded.funct7 == 0x04 {
                    // Zba/Zbb instructions (funct7 = 0x04)
                    match decoded.funct3 {
                        0x0 => Self::execute_sh1add(cpu, decoded),
                        0x1 => Self::execute_rol(cpu, decoded),
                        0x2 => Self::execute_sh2add(cpu, decoded),
                        0x3 => Self::execute_ror(cpu, decoded),
                        0x4 => Self::execute_sh3add(cpu, decoded),
                        0x7 => Self::execute_andn(cpu, decoded),
                        _ => {
                            result.success = false;
                            result.error_message = "Unknown Zba/Zbb func3".to_string();
                            result
                        }
                    }
                } else if decoded.funct7 == 0x05 {
                    // Zbb/Zbc instructions (funct7 = 0x05)
                    match decoded.funct3 {
                        0x0 => Self::execute_clz(cpu, decoded),
                        0x1 => Self::execute_ctz(cpu, decoded),
                        0x2 => Self::execute_cpop(cpu, decoded),
                        0x3 => Self::execute_clmul(cpu, decoded),
                        0x4 => Self::execute_clmulr(cpu, decoded),
                        0x5 => Self::execute_clmulh(cpu, decoded),
                        0x6 => Self::execute_min(cpu, decoded),
                        0x7 => Self::execute_max(cpu, decoded),
                        _ => {
                            result.success = false;
                            result.error_message = "Unknown Zbb/Zbc func3".to_string();
                            result
                        }
                    }
                } else if decoded.funct7 == 0x06 {
                    // Zbb additional instructions (funct7 = 0x06)
                    match decoded.funct3 {
                        0x0 => Self::execute_minu(cpu, decoded),
                        0x1 => Self::execute_maxu(cpu, decoded),
                        0x2 => Self::execute_min(cpu, decoded),
                        0x3 => Self::execute_max(cpu, decoded),
                        0x5 => Self::execute_orn(cpu, decoded),
                        0x6 => Self::execute_xorn(cpu, decoded),
                        _ => {
                            result.success = false;
                            result.error_message = "Unknown Zbb func3".to_string();
                            result
                        }
                    }
                } else if decoded.funct7 == 0x20 {
                    // Zba.UW instructions (funct7 = 0x20)
                    match decoded.funct3 {
                        0x0 => Self::execute_sh1add_uw(cpu, decoded),
                        0x2 => Self::execute_sh2add_uw(cpu, decoded),
                        0x4 => Self::execute_sh3add_uw(cpu, decoded),
                        _ => {
                            result.success = false;
                            result.error_message = "Unknown Zba.UW func3".to_string();
                            result
                        }
                    }
                } else if decoded.funct7 == 0x30 {
                    // Zba ADD.UW instruction
                    Self::execute_add_uw(cpu, decoded)
                } else {
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
            }

            instruction::OpCode::System => {
                if decoded.funct3 == 0x0 {
                    if decoded.imm == 0 { return Self::execute_ecall(cpu) }
                    if decoded.imm == 1 { return Self::execute_ebreak(cpu) }
                }
                // Zicsr extension: CSR instructions
                match decoded.funct3 {
                    0x1 => Self::execute_csrrw(cpu, decoded),
                    0x2 => Self::execute_csrrs(cpu, decoded),
                    0x3 => Self::execute_csrrc(cpu, decoded),
                    0x5 => Self::execute_csrrwi(cpu, decoded),
                    0x6 => Self::execute_csrrsi(cpu, decoded),
                    0x7 => Self::execute_csrrci(cpu, decoded),
                    _ => {
                        result.success = false;
                        result.error_message = "Unknown system instruction".to_string();
                        result
                    }
                }
            }

            instruction::OpCode::MiscMem => {
                // Zifencei extension: FENCE.I
                match decoded.funct3 {
                    0x0 => Self::execute_fence(cpu, decoded),
                    0x1 => Self::execute_fence_i(cpu, decoded),
                    _ => {
                        result.success = false;
                        result.error_message = "Unknown misc-mem instruction".to_string();
                        result
                    }
                }
            }

            instruction::OpCode::Atomic => {
                // AMO instructions: funct5 is in bits 31-27, stored in upper bits of funct7
                // funct7 format: funct5(5) | aq(1) | rl(1)
                let funct5 = (decoded.funct7 >> 2) & 0x1f;
                match funct5 {
                    0x02 => Self::execute_lr_d(cpu, decoded),      // LR.D
                    0x03 => Self::execute_sc_d(cpu, decoded),      // SC.D
                    0x00 => Self::execute_amoadd_d(cpu, decoded),  // AMOADD.D
                    0x01 => Self::execute_amoswap_d(cpu, decoded), // AMOSWAP.D
                    0x0c => Self::execute_amoand_d(cpu, decoded),  // AMOAND.D
                    0x0a => Self::execute_amoor_d(cpu, decoded),   // AMOOR.D
                    0x04 => Self::execute_amoxor_d(cpu, decoded),  // AMOXOR.D
                    0x18 => Self::execute_amomax_d(cpu, decoded),  // AMOMAX.D
                    0x1c => Self::execute_amomaxu_d(cpu, decoded), // AMOMAXU.D
                    0x10 => Self::execute_amomin_d(cpu, decoded),  // AMOMIN.D
                    0x14 => Self::execute_amominu_d(cpu, decoded), // AMOMINU.D
                    _ => {
                        result.success = false;
                        result.error_message = "Unknown AMO instruction".to_string();
                        result
                    }
                }
            }

            // Floating-point load instructions / 浮点加载指令
            instruction::OpCode::FpLoad => {
                match decoded.funct3 {
                    0x2 => Self::execute_flw(cpu, decoded),  // FLW (load single precision)
                    0x3 => Self::execute_fld(cpu, decoded),  // FLD (load double precision)
                    0x4 => Self::execute_flq(cpu, decoded),  // FLQ (load quad precision)
                    _ => {
                        result.success = false;
                        result.error_message = "Unknown FP load func3".to_string();
                        result
                    }
                }
            }

            // Floating-point store instructions / 浮点存储指令
            instruction::OpCode::FpStore => {
                match decoded.funct3 {
                    0x2 => Self::execute_fsw(cpu, decoded),  // FSW (store single precision)
                    0x3 => Self::execute_fsd(cpu, decoded),  // FSD (store double precision)
                    0x4 => Self::execute_fsq(cpu, decoded),  // FSQ (store quad precision)
                    _ => {
                        result.success = false;
                        result.error_message = "Unknown FP store func3".to_string();
                        result
                    }
                }
            }

            // Floating-point compute instructions / 浮点计算指令
            instruction::OpCode::FpCompute => {
                let funct5 = (decoded.funct7 >> 2) & 0x1f;
                let fmt = decoded.funct7 & 0x3;
                
                match funct5 {
                    0x00 => Self::execute_fp_add(cpu, decoded, fmt),         // FADD.S/D/Q
                    0x04 => Self::execute_fp_sub(cpu, decoded, fmt),         // FSUB.S/D/Q
                    0x08 => Self::execute_fp_mul(cpu, decoded, fmt),         // FMUL.S/D/Q
                    0x0c => Self::execute_fp_div(cpu, decoded, fmt),         // FDIV.S/D/Q
                    0x0b => Self::execute_fp_sqrt(cpu, decoded, fmt),        // FSQRT.S/D/Q
                    0x10 => Self::execute_fp_sgnj(cpu, decoded, fmt),        // FSGNJ.S/D/Q
                    0x05 => Self::execute_fp_minmax(cpu, decoded, fmt),      // FMIN/FMAX.S/D/Q
                    0x14 => Self::execute_fp_cmp(cpu, decoded, fmt),         // FLE/FLT/FEQ.S/D/Q
                    0x18 => Self::execute_fp_cvt_int(cpu, decoded, fmt),     // FCVT.W/L.S/D/Q
                    0x1c => Self::execute_fp_mvf_class(cpu, decoded, fmt),   // FMV.X.S/D/Q, FCLASS.S/D/Q
                    0x1e => Self::execute_fp_mvt(cpu, decoded, fmt),         // FMV.S/D/Q.X
                    _ => {
                        result.success = false;
                        result.error_message = format!("Unknown FP compute funct5: 0x{:02x}", funct5);
                        result
                    }
                }
            }

            // Vector instructions / 向量指令 (V 扩展)
            instruction::OpCode::Vector => {
                let funct6 = (decoded.funct7 >> 1) & 0x3f;
                let funct3 = decoded.funct3;
                
                match funct3 {
                    // OPIVV: Vector-Vector integer operations
                    0x0 => Self::execute_vector_opivv(cpu, decoded, funct6),
                    // OPFVV: Vector-Vector floating-point operations
                    0x1 => Self::execute_vector_opfvv(cpu, decoded, funct6),
                    // OPMVV: Vector multiply/divide operations
                    0x2 => Self::execute_vector_opmvv(cpu, decoded, funct6),
                    // OPVI: Vector-Immediate integer operations
                    0x3 => Self::execute_vector_opvi(cpu, decoded, funct6),
                    // OPIVI: Vector-5-bit immediate operations
                    0x4 => Self::execute_vector_opivi(cpu, decoded, funct6),
                    // OPFVF: Vector-Scalar FP operations
                    0x5 => Self::execute_vector_opfvf(cpu, decoded, funct6),
                    // VL/VS: Vector load/store
                    0x6 => Self::execute_vector_load(cpu, decoded, funct6),
                    // OPCFG: Vector configuration
                    0x7 => Self::execute_vector_config(cpu, decoded, funct6),
                    _ => {
                        result.success = false;
                        result.error_message = format!("Unknown vector funct3: 0x{:02x}", funct3);
                        result
                    }
                }
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
    // M Extension Instructions / M 扩展指令
    // ========================================

    /// MUL: Multiply low (乘法低位)
    ///
    /// rd = (rs1 * rs2)[127:0]
    fn execute_mul(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers_mut();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1) as u128;
        let rs2_val = Self::get_reg_value(regs, decoded.rs2) as u128;
        let rd_val = rs1_val.wrapping_mul(rs2_val);
        Self::set_reg_value(regs, decoded.rd, rd_val as i128);
        result
    }

    /// MULH: Multiply high signed (有符号乘法高位)
    ///
    /// rd = (rs1 * rs2)[255:128] (both signed)
    fn execute_mulh(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers_mut();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1);
        let rs2_val = Self::get_reg_value(regs, decoded.rs2);
        // Compute high 128 bits of signed multiplication
        let rd_val = Self::mulh_signed(rs1_val, rs2_val);
        Self::set_reg_value(regs, decoded.rd, rd_val);
        result
    }

    /// MULHSU: Multiply high signed-unsigned (有符号-无符号乘法高位)
    ///
    /// rd = (rs1 * rs2)[255:128] (rs1 signed, rs2 unsigned)
    fn execute_mulhsu(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers_mut();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1);
        let rs2_val = Self::get_reg_value(regs, decoded.rs2) as u128;
        // Compute high 128 bits of signed * unsigned multiplication
        let rd_val = Self::mulh_su(rs1_val, rs2_val);
        Self::set_reg_value(regs, decoded.rd, rd_val);
        result
    }

    /// MULHU: Multiply high unsigned (无符号乘法高位)
    ///
    /// rd = (rs1 * rs2)[255:128] (both unsigned)
    fn execute_mulhu(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers_mut();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1) as u128;
        let rs2_val = Self::get_reg_value(regs, decoded.rs2) as u128;
        // Compute high 128 bits of unsigned multiplication
        let rd_val = Self::mulh_unsigned(rs1_val, rs2_val);
        Self::set_reg_value(regs, decoded.rd, rd_val as i128);
        result
    }

    /// DIV: Divide signed (有符号除法)
    ///
    /// rd = rs1 / rs2 (signed)
    /// Special cases:
    /// - Division by zero: rd = -1
    /// - Overflow (INT128_MIN / -1): rd = INT128_MIN
    fn execute_div(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers_mut();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1);
        let rs2_val = Self::get_reg_value(regs, decoded.rs2);

        let rd_val = if rs2_val == 0 {
            // Division by zero returns -1
            -1i128
        } else if rs1_val == i128::MIN && rs2_val == -1 {
            // Overflow case: INT128_MIN / -1 = INT128_MIN
            i128::MIN
        } else {
            rs1_val / rs2_val
        };
        Self::set_reg_value(regs, decoded.rd, rd_val);
        result
    }

    /// DIVU: Divide unsigned (无符号除法)
    ///
    /// rd = rs1 / rs2 (unsigned)
    /// Special case: Division by zero returns 2^128 - 1
    fn execute_divu(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers_mut();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1) as u128;
        let rs2_val = Self::get_reg_value(regs, decoded.rs2) as u128;

        let rd_val = if rs2_val == 0 {
            // Division by zero returns 2^128 - 1 (all ones)
            u128::MAX
        } else {
            rs1_val / rs2_val
        };
        Self::set_reg_value(regs, decoded.rd, rd_val as i128);
        result
    }

    /// REM: Remainder signed (有符号取余)
    ///
    /// rd = rs1 % rs2 (signed)
    /// Special cases:
    /// - Division by zero: rd = rs1
    /// - Overflow (INT128_MIN % -1): rd = 0
    fn execute_rem(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers_mut();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1);
        let rs2_val = Self::get_reg_value(regs, decoded.rs2);

        let rd_val = if rs2_val == 0 {
            // Division by zero returns rs1
            rs1_val
        } else if rs1_val == i128::MIN && rs2_val == -1 {
            // Overflow case: INT128_MIN % -1 = 0
            0
        } else {
            rs1_val % rs2_val
        };
        Self::set_reg_value(regs, decoded.rd, rd_val);
        result
    }

    /// REMU: Remainder unsigned (无符号取余)
    ///
    /// rd = rs1 % rs2 (unsigned)
    /// Special case: Division by zero returns rs1
    fn execute_remu(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers_mut();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1) as u128;
        let rs2_val = Self::get_reg_value(regs, decoded.rs2) as u128;

        let rd_val = if rs2_val == 0 {
            // Division by zero returns rs1
            rs1_val
        } else {
            rs1_val % rs2_val
        };
        Self::set_reg_value(regs, decoded.rd, rd_val as i128);
        result
    }

    // ========================================
    // B Extension: Zba Instructions / B 扩展：Zba 指令（地址生成）
    // ========================================

    /// ADD.UW: Add unsigned word (无符号字加法)
    ///
    /// rd = rs1 + zero_extend(rs2[31:0])
    fn execute_add_uw(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1) as u128;
        let rs2_val = Self::get_reg_value(regs, decoded.rs2) as u128;
        
        // Zero extend lower 64 bits of rs2
        let rs2_zext = rs2_val & 0xFFFFFFFFFFFFFFFFu128;
        let rd_val = rs1_val.wrapping_add(rs2_zext);
        
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, rd_val as i128);
        result
    }

    /// SH1ADD: Shift left by 1 and add (左移1位后加法)
    ///
    /// rd = (rs1 << 1) + rs2
    fn execute_sh1add(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1) as u128;
        let rs2_val = Self::get_reg_value(regs, decoded.rs2) as u128;
        
        let rd_val = (rs1_val << 1).wrapping_add(rs2_val);
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, rd_val as i128);
        result
    }

    /// SH2ADD: Shift left by 2 and add (左移2位后加法)
    ///
    /// rd = (rs1 << 2) + rs2
    fn execute_sh2add(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1) as u128;
        let rs2_val = Self::get_reg_value(regs, decoded.rs2) as u128;
        
        let rd_val = (rs1_val << 2).wrapping_add(rs2_val);
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, rd_val as i128);
        result
    }

    /// SH3ADD: Shift left by 3 and add (左移3位后加法)
    ///
    /// rd = (rs1 << 3) + rs2
    fn execute_sh3add(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1) as u128;
        let rs2_val = Self::get_reg_value(regs, decoded.rs2) as u128;
        
        let rd_val = (rs1_val << 3).wrapping_add(rs2_val);
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, rd_val as i128);
        result
    }

    /// SH1ADD.UW: Shift left by 1 and add unsigned word (左移1位后无符号字加法)
    ///
    /// rd = (zero_extend(rs1[63:0]) << 1) + rs2
    fn execute_sh1add_uw(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1) as u128;
        let rs2_val = Self::get_reg_value(regs, decoded.rs2) as u128;
        
        // Zero extend lower 64 bits of rs1
        let rs1_zext = rs1_val & 0xFFFFFFFFFFFFFFFFu128;
        let rd_val = (rs1_zext << 1).wrapping_add(rs2_val);
        
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, rd_val as i128);
        result
    }

    /// SH2ADD.UW: Shift left by 2 and add unsigned word (左移2位后无符号字加法)
    ///
    /// rd = (zero_extend(rs1[63:0]) << 2) + rs2
    fn execute_sh2add_uw(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1) as u128;
        let rs2_val = Self::get_reg_value(regs, decoded.rs2) as u128;
        
        let rs1_zext = rs1_val & 0xFFFFFFFFFFFFFFFFu128;
        let rd_val = (rs1_zext << 2).wrapping_add(rs2_val);
        
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, rd_val as i128);
        result
    }

    /// SH3ADD.UW: Shift left by 3 and add unsigned word (左移3位后无符号字加法)
    ///
    /// rd = (zero_extend(rs1[63:0]) << 3) + rs2
    fn execute_sh3add_uw(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1) as u128;
        let rs2_val = Self::get_reg_value(regs, decoded.rs2) as u128;
        
        let rs1_zext = rs1_val & 0xFFFFFFFFFFFFFFFFu128;
        let rd_val = (rs1_zext << 3).wrapping_add(rs2_val);
        
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, rd_val as i128);
        result
    }

    /// SLLI.UW: Shift left logical immediate unsigned word (无符号字左移立即数)
    ///
    /// rd = zero_extend(rs1[63:0]) << shamt
    fn execute_slli_uw(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1) as u128;
        
        // Shift amount is in rs2 field for immediate instructions
        let shamt = decoded.rs2 as u32;
        
        // Zero extend lower 64 bits
        let rs1_zext = rs1_val & 0xFFFFFFFFFFFFFFFFu128;
        let rd_val = rs1_zext << shamt;
        
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, rd_val as i128);
        result
    }

    // ========================================
    // B Extension: Zbb Instructions / B 扩展：Zbb 指令（基本位操作）
    // ========================================

    /// ANDN: AND with inverted operand (带反转操作数的与)
    ///
    /// rd = rs1 & ~rs2
    fn execute_andn(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1) as u128;
        let rs2_val = Self::get_reg_value(regs, decoded.rs2) as u128;
        
        let rd_val = rs1_val & !rs2_val;
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, rd_val as i128);
        result
    }

    /// ORN: OR with inverted operand (带反转操作数的或)
    ///
    /// rd = rs1 | ~rs2
    fn execute_orn(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1) as u128;
        let rs2_val = Self::get_reg_value(regs, decoded.rs2) as u128;
        
        let rd_val = rs1_val | !rs2_val;
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, rd_val as i128);
        result
    }

    /// XORN: XOR with inverted operand (带反转操作数的异或)
    ///
    /// rd = rs1 ^ ~rs2
    fn execute_xorn(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1) as u128;
        let rs2_val = Self::get_reg_value(regs, decoded.rs2) as u128;
        
        let rd_val = rs1_val ^ !rs2_val;
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, rd_val as i128);
        result
    }

    /// CLZ: Count leading zeros (前导零计数)
    ///
    /// rd = number of leading zeros in rs1
    fn execute_clz(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1) as u128;
        
        let rd_val = rs1_val.leading_zeros() as u128;
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, rd_val as i128);
        result
    }

    /// CTZ: Count trailing zeros (尾随零计数)
    ///
    /// rd = number of trailing zeros in rs1
    fn execute_ctz(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1) as u128;
        
        let rd_val = rs1_val.trailing_zeros() as u128;
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, rd_val as i128);
        result
    }

    /// CPOP: Population count (置位计数)
    ///
    /// rd = number of 1 bits in rs1
    fn execute_cpop(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1) as u128;
        
        let rd_val = rs1_val.count_ones() as u128;
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, rd_val as i128);
        result
    }

    /// MAX: Maximum signed (有符号最大值)
    ///
    /// rd = max(rs1, rs2) (signed comparison)
    fn execute_max(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1);
        let rs2_val = Self::get_reg_value(regs, decoded.rs2);
        
        let rd_val = std::cmp::max(rs1_val, rs2_val);
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, rd_val);
        result
    }

    /// MAXU: Maximum unsigned (无符号最大值)
    ///
    /// rd = max(rs1, rs2) (unsigned comparison)
    fn execute_maxu(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1) as u128;
        let rs2_val = Self::get_reg_value(regs, decoded.rs2) as u128;
        
        let rd_val = std::cmp::max(rs1_val, rs2_val);
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, rd_val as i128);
        result
    }

    /// MIN: Minimum signed (有符号最小值)
    ///
    /// rd = min(rs1, rs2) (signed comparison)
    fn execute_min(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1);
        let rs2_val = Self::get_reg_value(regs, decoded.rs2);
        
        let rd_val = std::cmp::min(rs1_val, rs2_val);
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, rd_val);
        result
    }

    /// MINU: Minimum unsigned (无符号最小值)
    ///
    /// rd = min(rs1, rs2) (unsigned comparison)
    fn execute_minu(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1) as u128;
        let rs2_val = Self::get_reg_value(regs, decoded.rs2) as u128;
        
        let rd_val = std::cmp::min(rs1_val, rs2_val);
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, rd_val as i128);
        result
    }

    /// ROL: Rotate left (循环左移)
    ///
    /// rd = (rs1 << (rs2 & 0x7f)) | (rs1 >> (128 - (rs2 & 0x7f)))
    fn execute_rol(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1) as u128;
        let rs2_val = Self::get_reg_value(regs, decoded.rs2) as u128;
        
        let shamt = (rs2_val & 0x7f) as u32;
        let rd_val = rs1_val.rotate_left(shamt);
        
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, rd_val as i128);
        result
    }

    /// ROR: Rotate right (循环右移)
    ///
    /// rd = (rs1 >> (rs2 & 0x7f)) | (rs1 << (128 - (rs2 & 0x7f)))
    fn execute_ror(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1) as u128;
        let rs2_val = Self::get_reg_value(regs, decoded.rs2) as u128;
        
        let shamt = (rs2_val & 0x7f) as u32;
        let rd_val = rs1_val.rotate_right(shamt);
        
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, rd_val as i128);
        result
    }

    /// RORI: Rotate right immediate (循环右移立即数)
    ///
    /// rd = (rs1 >> shamt) | (rs1 << (128 - shamt))
    fn execute_rori(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1) as u128;
        
        // Shift amount is in rs2 field
        let shamt = decoded.rs2 as u32;
        let rd_val = rs1_val.rotate_right(shamt);
        
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, rd_val as i128);
        result
    }

    /// BSETI: Bit set immediate (位设置立即数)
    ///
    /// rd = rs1 | (1 << shamt)
    fn execute_bseti(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1) as u128;
        
        let shamt = decoded.rs2 as u32;
        let rd_val = rs1_val | (1u128 << shamt);
        
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, rd_val as i128);
        result
    }

    /// BCLRI: Bit clear immediate (位清除立即数)
    ///
    /// rd = rs1 & ~(1 << shamt)
    fn execute_bclri(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1) as u128;
        
        let shamt = decoded.rs2 as u32;
        let rd_val = rs1_val & !(1u128 << shamt);
        
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, rd_val as i128);
        result
    }

    /// BINVI: Bit invert immediate (位反转立即数)
    ///
    /// rd = rs1 ^ (1 << shamt)
    fn execute_binvi(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1) as u128;
        
        let shamt = decoded.rs2 as u32;
        let rd_val = rs1_val ^ (1u128 << shamt);
        
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, rd_val as i128);
        result
    }

    /// BEXTI: Bit extract immediate (位提取立即数)
    ///
    /// rd = (rs1 >> shamt) & 1
    fn execute_bexti(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1) as u128;
        
        let shamt = decoded.rs2 as u32;
        let rd_val = (rs1_val >> shamt) & 1;
        
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, rd_val as i128);
        result
    }

    // ========================================
    // B Extension: Zbc Instructions / B 扩展：Zbc 指令（进位乘法）
    // ========================================

    /// CLMUL: Carry-less multiply (无进位乘法)
    ///
    /// Computes the carry-less product of rs1 and rs2.
    /// rd = clmul(rs1, rs2) = polynomial multiplication over GF(2)
    fn execute_clmul(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1) as u128;
        let rs2_val = Self::get_reg_value(regs, decoded.rs2) as u128;
        
        let rd_val = Self::clmul(rs1_val, rs2_val);
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, rd_val as i128);
        result
    }

    /// CLMULH: Carry-less multiply high (无进位乘法高位)
    ///
    /// Returns the high 128 bits of the carry-less product.
    fn execute_clmulh(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1) as u128;
        let rs2_val = Self::get_reg_value(regs, decoded.rs2) as u128;
        
        let rd_val = Self::clmulh(rs1_val, rs2_val);
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, rd_val as i128);
        result
    }

    /// CLMULR: Carry-less multiply reversed (无进位乘法反转)
    ///
    /// Returns bits [127:64] of the carry-less product.
    fn execute_clmulr(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1) as u128;
        let rs2_val = Self::get_reg_value(regs, decoded.rs2) as u128;
        
        let rd_val = Self::clmulr(rs1_val, rs2_val);
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, rd_val as i128);
        result
    }

    // ========================================
    // Helper functions for M extension / M 扩展辅助函数
    // ========================================

    /// Computes high 128 bits of unsigned 128x128 multiplication.
    /// 
    /// 计算无符号 128x128 乘法的高 128 位。
    fn mulh_unsigned(a: u128, b: u128) -> u128 {
        // Split into high and low 64-bit parts
        let a_lo = a as u64 as u128;
        let a_hi = (a >> 64) as u64 as u128;
        let b_lo = b as u64 as u128;
        let b_hi = (b >> 64) as u64 as u128;

        // Compute partial products
        let lo_lo = a_lo * b_lo;
        let hi_lo = a_hi * b_lo;
        let lo_hi = a_lo * b_hi;
        let hi_hi = a_hi * b_hi;

        // Compute the middle sum with carry
        let mid = (lo_lo >> 64) + (hi_lo & ((1u128 << 64) - 1)) + (lo_hi & ((1u128 << 64) - 1));
        let carry = (mid >> 64) + (hi_lo >> 64) + (lo_hi >> 64);

        // High 128 bits
        hi_hi + carry
    }

    /// Computes high 128 bits of signed 128x128 multiplication.
    /// 
    /// 计算有符号 128x128 乘法的高 128 位。
    fn mulh_signed(a: i128, b: i128) -> i128 {
        // Convert to unsigned, compute unsigned high bits
        let a_unsigned = a as u128;
        let b_unsigned = b as u128;
        
        let mut result = Self::mulh_unsigned(a_unsigned, b_unsigned);
        
        // Adjust for negative operands
        if a < 0 {
            result = result.wrapping_sub(b_unsigned);
        }
        if b < 0 {
            result = result.wrapping_sub(a_unsigned);
        }
        
        result as i128
    }

    /// Computes high 128 bits of signed * unsigned 128x128 multiplication.
    /// 
    /// 计算有符号 * 无符号 128x128 乘法的高 128 位。
    fn mulh_su(a: i128, b: u128) -> i128 {
        let a_unsigned = a as u128;
        
        let mut result = Self::mulh_unsigned(a_unsigned, b);
        
        // Adjust for negative signed operand
        if a < 0 {
            result = result.wrapping_sub(b);
        }
        
        result as i128
    }

    // ========================================
    // Helper functions for B extension (clmul) / B 扩展辅助函数
    // ========================================

    /// Carry-less multiplication - returns low 128 bits of result.
    /// 
    /// 无进位乘法 - 返回结果低 128 位。
    fn clmul(a: u128, b: u128) -> u128 {
        let mut result = 0u128;
        let mut b_temp = b;
        let mut i = 0u32;
        
        while b_temp != 0 {
            if (b_temp & 1) != 0 {
                result ^= a << i;
            }
            b_temp >>= 1;
            i += 1;
        }
        
        result
    }

    /// Carry-less multiplication - returns high 128 bits of result.
    /// 
    /// 无进位乘法 - 返回结果高 128 位。
    fn clmulh(a: u128, b: u128) -> u128 {
        let mut result = 0u128;
        let mut b_temp = b;
        let mut i = 0u32;
        
        while b_temp != 0 {
            if (b_temp & 1) != 0 {
                // Only add to result if shift >= 128
                if i >= 128 {
                    result ^= a << (i - 128);
                }
            }
            b_temp >>= 1;
            i += 1;
        }
        
        result
    }

    /// Carry-less multiplication - returns bits [127:64] (equivalent to clmulr in spec).
    /// 
    /// 无进位乘法 - 返回位 [127:64]。
    fn clmulr(a: u128, b: u128) -> u128 {
        // CLMULR returns bits [X-1:X/2-1] for X-bit registers
        // For 128-bit, this is bits [127:64]
        // This is equivalent to (clmul(a, b) >> 64) & 0xFFFFFFFFFFFFFFFF
        Self::clmul(a, b) >> 64
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

    // ========================================
    // Zicsr Extension Instructions (CSR) / Zicsr 扩展指令（CSR）
    // ========================================

    /// CSRRW: Atomic Read/Write CSR (原子读/写 CSR)
    ///
    /// t = CSR[csr]; CSR[csr] = rs1; rd = t
    fn execute_csrrw(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let csr_addr = register::CsrAddress::from_u16(decoded.imm as u16);
        
        // Get rs1 value
        let rs1_val = Self::get_reg_value(cpu.get_registers(), decoded.rs1);
        
        // Read old CSR value
        let old_val = cpu.get_registers().read_csr(csr_addr);
        
        // Write new CSR value
        cpu.get_registers_mut().write_csr(csr_addr, rs1_val as u128);
        
        // Write old value to rd
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, old_val as i128);
        
        result
    }

    /// CSRRS: Atomic Read and Set Bits in CSR (原子读并设置 CSR 位)
    ///
    /// t = CSR[csr]; CSR[csr] = CSR[csr] | rs1; rd = t
    fn execute_csrrs(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let csr_addr = register::CsrAddress::from_u16(decoded.imm as u16);
        
        // Get rs1 value (mask of bits to set)
        let rs1_val = Self::get_reg_value(cpu.get_registers(), decoded.rs1) as u128;
        
        // Read old CSR value
        let old_val = cpu.get_registers().read_csr(csr_addr);
        
        // Set bits (if rs1 != 0)
        if decoded.rs1 != 0 {
            let new_val = old_val | rs1_val;
            cpu.get_registers_mut().write_csr(csr_addr, new_val);
        }
        
        // Write old value to rd
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, old_val as i128);
        
        result
    }

    /// CSRRC: Atomic Read and Clear Bits in CSR (原子读并清除 CSR 位)
    ///
    /// t = CSR[csr]; CSR[csr] = CSR[csr] & ~rs1; rd = t
    fn execute_csrrc(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let csr_addr = register::CsrAddress::from_u16(decoded.imm as u16);
        
        // Get rs1 value (mask of bits to clear)
        let rs1_val = Self::get_reg_value(cpu.get_registers(), decoded.rs1) as u128;
        
        // Read old CSR value
        let old_val = cpu.get_registers().read_csr(csr_addr);
        
        // Clear bits (if rs1 != 0)
        if decoded.rs1 != 0 {
            let new_val = old_val & !rs1_val;
            cpu.get_registers_mut().write_csr(csr_addr, new_val);
        }
        
        // Write old value to rd
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, old_val as i128);
        
        result
    }

    /// CSRRWI: Atomic Read/Write CSR Immediate (原子读/写 CSR 立即数)
    ///
    /// t = CSR[csr]; CSR[csr] = zimm; rd = t
    fn execute_csrrwi(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let csr_addr = register::CsrAddress::from_u16(decoded.imm as u16);
        
        // Get immediate value (zimm is in rs1 field, zero-extended)
        let zimm = decoded.rs1 as u128;
        
        // Read old CSR value
        let old_val = cpu.get_registers().read_csr(csr_addr);
        
        // Write new CSR value
        cpu.get_registers_mut().write_csr(csr_addr, zimm);
        
        // Write old value to rd
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, old_val as i128);
        
        result
    }

    /// CSRRSI: Atomic Read and Set Bits in CSR Immediate (原子读并设置 CSR 位立即数)
    ///
    /// t = CSR[csr]; CSR[csr] = CSR[csr] | zimm; rd = t
    fn execute_csrrsi(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let csr_addr = register::CsrAddress::from_u16(decoded.imm as u16);
        
        // Get immediate value (zimm is in rs1 field)
        let zimm = decoded.rs1 as u128;
        
        // Read old CSR value
        let old_val = cpu.get_registers().read_csr(csr_addr);
        
        // Set bits (if zimm != 0)
        if zimm != 0 {
            let new_val = old_val | zimm;
            cpu.get_registers_mut().write_csr(csr_addr, new_val);
        }
        
        // Write old value to rd
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, old_val as i128);
        
        result
    }

    /// CSRRCI: Atomic Read and Clear Bits in CSR Immediate (原子读并清除 CSR 位立即数)
    ///
    /// t = CSR[csr]; CSR[csr] = CSR[csr] & ~zimm; rd = t
    fn execute_csrrci(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let csr_addr = register::CsrAddress::from_u16(decoded.imm as u16);
        
        // Get immediate value (zimm is in rs1 field)
        let zimm = decoded.rs1 as u128;
        
        // Read old CSR value
        let old_val = cpu.get_registers().read_csr(csr_addr);
        
        // Clear bits (if zimm != 0)
        if zimm != 0 {
            let new_val = old_val & !zimm;
            cpu.get_registers_mut().write_csr(csr_addr, new_val);
        }
        
        // Write old value to rd
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, old_val as i128);
        
        result
    }

    // ========================================
    // Zifencei Extension Instructions / Zifencei 扩展指令
    // ========================================

    /// FENCE: Memory fence (内存屏障)
    ///
    /// Orders memory operations.
    fn execute_fence(_cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        // In a simple implementation, FENCE is a no-op
        // A real implementation would flush/store buffers and ensure ordering
        // pred (predecessor) = bits [27:24] of instruction
        // succ (successor) = bits [23:20] of instruction
        let _pred = (decoded.funct7 >> 4) & 0xf;
        let _succ = decoded.funct7 & 0xf;
        // For now, just ensure all memory operations are visible
        // This is a no-op in our simple VM
        result
    }

    /// FENCE.I: Instruction fence (指令屏障)
    ///
    /// Ensures instruction fetches see previous stores.
    fn execute_fence_i(_cpu: &mut cpu::CPU, _decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        // In a simple implementation, FENCE.I is a no-op
        // A real implementation would flush the instruction cache
        // For our simple VM, this is a no-op since we don't have caches
        result
    }

    // ========================================
    // P Extension Instructions (SIMD) / P 扩展指令（SIMD）
    // ========================================

    /// Execute SIMD instructions for P extension.
    ///
    /// SIMD instructions perform parallel operations on sub-elements.
    ///
    /// ---
    ///
    /// 执行 P 扩展的 SIMD 指令。
    ///
    /// SIMD 指令对子元素执行并行操作。
    fn execute_simd(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let op = decoded.funct7 & 0x7f;
        let funct3 = decoded.funct3;
        
        let regs = cpu.get_registers();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1) as u128;
        let rs2_val = Self::get_reg_value(regs, decoded.rs2) as u128;
        
        // Determine element size from funct3
        let element_size = match funct3 {
            0x0 => 8,   // 8-bit elements
            0x1 => 16,  // 16-bit elements
            0x2 => 32,  // 32-bit elements
            0x3 => 64,  // 64-bit elements
            _ => return result, // Unknown size
        };
        
        let rd_val = match op {
            // SIMD Addition
            0x00 => Self::simd_add(rs1_val, rs2_val, element_size),
            // SIMD Subtraction
            0x02 => Self::simd_sub(rs1_val, rs2_val, element_size),
            // SIMD Unsigned Addition with saturation
            0x04 => Self::simd_addu(rs1_val, rs2_val, element_size),
            // SIMD Signed Addition with saturation
            0x05 => Self::simd_adds(rs1_val, rs2_val, element_size),
            // SIMD Unsigned Subtraction with saturation
            0x06 => Self::simd_subu(rs1_val, rs2_val, element_size),
            // SIMD Signed Subtraction with saturation
            0x07 => Self::simd_subs(rs1_val, rs2_val, element_size),
            // SIMD Shift Left
            0x0a => Self::simd_sll(rs1_val, rs2_val, element_size),
            // SIMD Shift Right Logical
            0x0c => Self::simd_srl(rs1_val, rs2_val, element_size),
            // SIMD Shift Right Arithmetic
            0x0d => Self::simd_sra(rs1_val, rs2_val, element_size),
            // SIMD AND
            0x18 => Self::simd_and(rs1_val, rs2_val, element_size),
            // SIMD OR
            0x1a => Self::simd_or(rs1_val, rs2_val, element_size),
            // SIMD XOR
            0x1c => Self::simd_xor(rs1_val, rs2_val, element_size),
            // SIMD Pack
            0x20 => Self::simd_pack(rs1_val, rs2_val, element_size),
            // SIMD Pack Upper
            0x21 => Self::simd_packu(rs1_val, rs2_val, element_size),
            // SIMD Multiply
            0x24 => Self::simd_mul(rs1_val, rs2_val, element_size),
            _ => rs1_val, // Unknown operation, return rs1
        };
        
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, rd_val as i128);
        result
    }

    // SIMD helper functions / SIMD 辅助函数

    /// SIMD addition without saturation.
    fn simd_add(rs1: u128, rs2: u128, element_size: u32) -> u128 {
        let mask = (1u128 << element_size) - 1;
        let num_elements = 128 / element_size;
        let mut result = 0u128;
        
        for i in 0..num_elements {
            let shift = i * element_size;
            let a = (rs1 >> shift) & mask;
            let b = (rs2 >> shift) & mask;
            let sum = a.wrapping_add(b) & mask;
            result |= sum << shift;
        }
        result
    }

    /// SIMD subtraction without saturation.
    fn simd_sub(rs1: u128, rs2: u128, element_size: u32) -> u128 {
        let mask = (1u128 << element_size) - 1;
        let num_elements = 128 / element_size;
        let mut result = 0u128;
        
        for i in 0..num_elements {
            let shift = i * element_size;
            let a = (rs1 >> shift) & mask;
            let b = (rs2 >> shift) & mask;
            let diff = a.wrapping_sub(b) & mask;
            result |= diff << shift;
        }
        result
    }

    /// SIMD unsigned addition with saturation.
    fn simd_addu(rs1: u128, rs2: u128, element_size: u32) -> u128 {
        let mask = (1u128 << element_size) - 1;
        let num_elements = 128 / element_size;
        let mut result = 0u128;
        
        for i in 0..num_elements {
            let shift = i * element_size;
            let a = (rs1 >> shift) & mask;
            let b = (rs2 >> shift) & mask;
            let sum = a.saturating_add(b);
            result |= (sum & mask) << shift;
        }
        result
    }

    /// SIMD signed addition with saturation.
    fn simd_adds(rs1: u128, rs2: u128, element_size: u32) -> u128 {
        let mask = (1u128 << element_size) - 1;
        let sign_bit = 1u128 << (element_size - 1);
        let num_elements = 128 / element_size;
        let mut result = 0u128;
        
        for i in 0..num_elements {
            let shift = i * element_size;
            let a = ((rs1 >> shift) & mask) as i128;
            let b = ((rs2 >> shift) & mask) as i128;
            
            // Convert to signed
            let a_signed = if a & (sign_bit as i128) != 0 { a - ((mask + 1) as i128) } else { a };
            let b_signed = if b & (sign_bit as i128) != 0 { b - ((mask + 1) as i128) } else { b };
            
            // Saturating add
            let max = (mask >> 1) as i128;
            let min = -(max + 1);
            let sum = a_signed.saturating_add(b_signed);
            let sum = if sum > max { max } else if sum < min { min } else { sum };
            let sum = if sum < 0 { (sum + (mask as i128 + 1)) as u128 } else { sum as u128 };
            result |= (sum & mask) << shift;
        }
        result
    }

    /// SIMD unsigned subtraction with saturation.
    fn simd_subu(rs1: u128, rs2: u128, element_size: u32) -> u128 {
        let mask = (1u128 << element_size) - 1;
        let num_elements = 128 / element_size;
        let mut result = 0u128;
        
        for i in 0..num_elements {
            let shift = i * element_size;
            let a = (rs1 >> shift) & mask;
            let b = (rs2 >> shift) & mask;
            let diff = a.saturating_sub(b);
            result |= (diff & mask) << shift;
        }
        result
    }

    /// SIMD signed subtraction with saturation.
    fn simd_subs(rs1: u128, rs2: u128, element_size: u32) -> u128 {
        let mask = (1u128 << element_size) - 1;
        let sign_bit = 1u128 << (element_size - 1);
        let num_elements = 128 / element_size;
        let mut result = 0u128;
        
        for i in 0..num_elements {
            let shift = i * element_size;
            let a = ((rs1 >> shift) & mask) as i128;
            let b = ((rs2 >> shift) & mask) as i128;
            
            // Convert to signed
            let a_signed = if a & (sign_bit as i128) != 0 { a - ((mask + 1) as i128) } else { a };
            let b_signed = if b & (sign_bit as i128) != 0 { b - ((mask + 1) as i128) } else { b };
            
            // Saturating sub
            let max = (mask >> 1) as i128;
            let min = -(max + 1);
            let diff = a_signed.saturating_sub(b_signed);
            let diff = if diff > max { max } else if diff < min { min } else { diff };
            let diff = if diff < 0 { (diff + (mask as i128 + 1)) as u128 } else { diff as u128 };
            result |= (diff & mask) << shift;
        }
        result
    }

    /// SIMD shift left logical.
    fn simd_sll(rs1: u128, rs2: u128, element_size: u32) -> u128 {
        let mask = (1u128 << element_size) - 1;
        let shift_mask = element_size - 1;
        let num_elements = 128 / element_size;
        let mut result = 0u128;
        
        for i in 0..num_elements {
            let shift = i * element_size;
            let a = (rs1 >> shift) & mask;
            let b = (rs2 >> shift) & mask;
            let shift_amount = (b & shift_mask as u128) as u32;
            let shifted = (a << shift_amount) & mask;
            result |= shifted << shift;
        }
        result
    }

    /// SIMD shift right logical.
    fn simd_srl(rs1: u128, rs2: u128, element_size: u32) -> u128 {
        let mask = (1u128 << element_size) - 1;
        let shift_mask = element_size - 1;
        let num_elements = 128 / element_size;
        let mut result = 0u128;
        
        for i in 0..num_elements {
            let shift = i * element_size;
            let a = (rs1 >> shift) & mask;
            let b = (rs2 >> shift) & mask;
            let shift_amount = (b & shift_mask as u128) as u32;
            let shifted = (a >> shift_amount) & mask;
            result |= shifted << shift;
        }
        result
    }

    /// SIMD shift right arithmetic.
    fn simd_sra(rs1: u128, rs2: u128, element_size: u32) -> u128 {
        let mask = (1u128 << element_size) - 1;
        let sign_bit = 1u128 << (element_size - 1);
        let shift_mask = element_size - 1;
        let num_elements = 128 / element_size;
        let mut result = 0u128;
        
        for i in 0..num_elements {
            let shift = i * element_size;
            let a = (rs1 >> shift) & mask;
            let b = (rs2 >> shift) & mask;
            let shift_amount = (b & shift_mask as u128) as u32;
            
            // Sign extend if negative
            let shifted = if a & sign_bit != 0 {
                // Negative number: fill with 1s
                let sign_ext = mask << (element_size - shift_amount);
                ((a >> shift_amount) | sign_ext) & mask
            } else {
                (a >> shift_amount) & mask
            };
            result |= shifted << shift;
        }
        result
    }

    /// SIMD bitwise AND.
    fn simd_and(rs1: u128, rs2: u128, _element_size: u32) -> u128 {
        rs1 & rs2
    }

    /// SIMD bitwise OR.
    fn simd_or(rs1: u128, rs2: u128, _element_size: u32) -> u128 {
        rs1 | rs2
    }

    /// SIMD bitwise XOR.
    fn simd_xor(rs1: u128, rs2: u128, _element_size: u32) -> u128 {
        rs1 ^ rs2
    }

    /// SIMD pack lower halves.
    fn simd_pack(rs1: u128, rs2: u128, element_size: u32) -> u128 {
        let half_mask = (1u128 << (element_size / 2)) - 1;
        let num_elements = 128 / element_size;
        let mut result = 0u128;
        
        for i in 0..num_elements {
            let shift = i * element_size;
            
            if i < num_elements / 2 {
                // Lower half from rs1
                let a = (rs1 >> (i * element_size)) & half_mask;
                result |= a << shift;
            } else {
                // Lower half from rs2
                let j = i - num_elements / 2;
                let b = (rs2 >> (j * element_size)) & half_mask;
                result |= b << shift;
            }
        }
        result
    }

    /// SIMD pack upper halves.
    fn simd_packu(rs1: u128, rs2: u128, element_size: u32) -> u128 {
        let half_mask = (1u128 << (element_size / 2)) - 1;
        let num_elements = 128 / element_size;
        let mut result = 0u128;
        
        for i in 0..num_elements {
            let shift = i * element_size;
            
            if i < num_elements / 2 {
                // Upper half from rs1
                let a = (rs1 >> (i * element_size + element_size / 2)) & half_mask;
                result |= a << shift;
            } else {
                // Upper half from rs2
                let j = i - num_elements / 2;
                let b = (rs2 >> (j * element_size + element_size / 2)) & half_mask;
                result |= b << shift;
            }
        }
        result
    }

    /// SIMD multiplication.
    fn simd_mul(rs1: u128, rs2: u128, element_size: u32) -> u128 {
        let mask = (1u128 << element_size) - 1;
        let num_elements = 128 / element_size;
        let mut result = 0u128;
        
        for i in 0..num_elements {
            let shift = i * element_size;
            let a = (rs1 >> shift) & mask;
            let b = (rs2 >> shift) & mask;
            let product = (a.wrapping_mul(b)) & mask;
            result |= product << shift;
        }
        result
    }

    // ========================================
    // A Extension Instructions (Atomic) / A 扩展指令（原子）
    // ========================================

    /// LR.D: Load Reserved Doubleword (加载保留双字)
    ///
    /// rd = MEM[rs1]; creates a reservation at rs1
    fn execute_lr_d(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1);
        let addr = rs1_val as memory::Address128;
        
        let rd_val = cpu.get_memory().borrow_mut().load_reserved_128(addr);
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, rd_val as i128);
        result
    }

    /// SC.D: Store Conditional Doubleword (存储条件双字)
    ///
    /// If reservation is valid: MEM[rs1] = rs2; rd = 0
    /// If reservation is invalid: rd = 1
    fn execute_sc_d(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1);
        let rs2_val = Self::get_reg_value(regs, decoded.rs2);
        let addr = rs1_val as memory::Address128;
        
        let sc_result = cpu.get_memory().borrow_mut().store_conditional_128(addr, rs2_val as memory::Word128);
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, sc_result as i128);
        result
    }

    /// AMOADD.D: Atomic Memory Add Doubleword (原子内存加法双字)
    ///
    /// rd = MEM[rs1]; MEM[rs1] = MEM[rs1] + rs2
    fn execute_amoadd_d(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1);
        let rs2_val = Self::get_reg_value(regs, decoded.rs2);
        let addr = rs1_val as memory::Address128;
        
        let original = cpu.get_memory().borrow_mut().amo_add_128(addr, rs2_val as memory::Word128);
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, original as i128);
        result
    }

    /// AMOSWAP.D: Atomic Memory Swap Doubleword (原子内存交换双字)
    ///
    /// rd = MEM[rs1]; MEM[rs1] = rs2
    fn execute_amoswap_d(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1);
        let rs2_val = Self::get_reg_value(regs, decoded.rs2);
        let addr = rs1_val as memory::Address128;
        
        let original = cpu.get_memory().borrow_mut().amo_swap_128(addr, rs2_val as memory::Word128);
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, original as i128);
        result
    }

    /// AMOAND.D: Atomic Memory AND Doubleword (原子内存与双字)
    ///
    /// rd = MEM[rs1]; MEM[rs1] = MEM[rs1] & rs2
    fn execute_amoand_d(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1);
        let rs2_val = Self::get_reg_value(regs, decoded.rs2);
        let addr = rs1_val as memory::Address128;
        
        let original = cpu.get_memory().borrow_mut().amo_and_128(addr, rs2_val as memory::Word128);
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, original as i128);
        result
    }

    /// AMOOR.D: Atomic Memory OR Doubleword (原子内存或双字)
    ///
    /// rd = MEM[rs1]; MEM[rs1] = MEM[rs1] | rs2
    fn execute_amoor_d(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1);
        let rs2_val = Self::get_reg_value(regs, decoded.rs2);
        let addr = rs1_val as memory::Address128;
        
        let original = cpu.get_memory().borrow_mut().amo_or_128(addr, rs2_val as memory::Word128);
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, original as i128);
        result
    }

    /// AMOXOR.D: Atomic Memory XOR Doubleword (原子内存异或双字)
    ///
    /// rd = MEM[rs1]; MEM[rs1] = MEM[rs1] ^ rs2
    fn execute_amoxor_d(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1);
        let rs2_val = Self::get_reg_value(regs, decoded.rs2);
        let addr = rs1_val as memory::Address128;
        
        let original = cpu.get_memory().borrow_mut().amo_xor_128(addr, rs2_val as memory::Word128);
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, original as i128);
        result
    }

    /// AMOMAX.D: Atomic Memory Maximum Doubleword (signed) (原子内存有符号最大值双字)
    ///
    /// rd = MEM[rs1]; MEM[rs1] = max(MEM[rs1], rs2) (signed)
    fn execute_amomax_d(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1);
        let rs2_val = Self::get_reg_value(regs, decoded.rs2);
        let addr = rs1_val as memory::Address128;
        
        let original = cpu.get_memory().borrow_mut().amo_max_128(addr, rs2_val as memory::Word128);
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, original as i128);
        result
    }

    /// AMOMAXU.D: Atomic Memory Maximum Doubleword (unsigned) (原子内存无符号最大值双字)
    ///
    /// rd = MEM[rs1]; MEM[rs1] = max(MEM[rs1], rs2) (unsigned)
    fn execute_amomaxu_d(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1);
        let rs2_val = Self::get_reg_value(regs, decoded.rs2);
        let addr = rs1_val as memory::Address128;
        
        let original = cpu.get_memory().borrow_mut().amo_maxu_128(addr, rs2_val as memory::Word128);
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, original as i128);
        result
    }

    /// AMOMIN.D: Atomic Memory Minimum Doubleword (signed) (原子内存有符号最小值双字)
    ///
    /// rd = MEM[rs1]; MEM[rs1] = min(MEM[rs1], rs2) (signed)
    fn execute_amomin_d(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1);
        let rs2_val = Self::get_reg_value(regs, decoded.rs2);
        let addr = rs1_val as memory::Address128;
        
        let original = cpu.get_memory().borrow_mut().amo_min_128(addr, rs2_val as memory::Word128);
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, original as i128);
        result
    }

    /// AMOMINU.D: Atomic Memory Minimum Doubleword (unsigned) (原子内存无符号最小值双字)
    ///
    /// rd = MEM[rs1]; MEM[rs1] = min(MEM[rs1], rs2) (unsigned)
    fn execute_amominu_d(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1);
        let rs2_val = Self::get_reg_value(regs, decoded.rs2);
        let addr = rs1_val as memory::Address128;
        
        let original = cpu.get_memory().borrow_mut().amo_minu_128(addr, rs2_val as memory::Word128);
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, original as i128);
        result
    }

    // ========================================
    // F/D/Q Extension Instructions / F/D/Q 扩展指令
    // ========================================

    /// Helper: Get floating-point register value as f32 (single precision).
    ///
    /// ---
    ///
    /// 辅助函数：获取浮点寄存器值作为 f32（单精度）。
    fn get_fp_reg_f32(regs: &register::Register, reg: u8) -> f32 {
        let fp_index = unsafe { std::mem::transmute::<u8, register::FRegisterIndex>(reg) };
        let bits = regs.read_fp(fp_index) as u32;
        f32::from_bits(bits)
    }

    /// Helper: Get floating-point register value as f64 (double precision).
    ///
    /// ---
    ///
    /// 辅助函数：获取浮点寄存器值作为 f64（双精度）。
    fn get_fp_reg_f64(regs: &register::Register, reg: u8) -> f64 {
        let fp_index = unsafe { std::mem::transmute::<u8, register::FRegisterIndex>(reg) };
        let bits = regs.read_fp(fp_index) as u64;
        f64::from_bits(bits)
    }

    /// Helper: Set floating-point register value from f32.
    ///
    /// ---
    ///
    /// 辅助函数：从 f32 设置浮点寄存器值。
    fn set_fp_reg_f32(regs: &mut register::Register, reg: u8, value: f32) {
        let fp_index = unsafe { std::mem::transmute::<u8, register::FRegisterIndex>(reg) };
        let bits = value.to_bits() as u128;
        regs.write_fp(fp_index, bits);
    }

    /// Helper: Set floating-point register value from f64.
    ///
    /// ---
    ///
    /// 辅助函数：从 f64 设置浮点寄存器值。
    fn set_fp_reg_f64(regs: &mut register::Register, reg: u8, value: f64) {
        let fp_index = unsafe { std::mem::transmute::<u8, register::FRegisterIndex>(reg) };
        let bits = value.to_bits() as u128;
        regs.write_fp(fp_index, bits);
    }

    /// FLW: Load Single-Precision Floating-Point (加载单精度浮点数)
    ///
    /// fd = MEM[rs1 + imm][31:0]
    fn execute_flw(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1);
        let addr = (rs1_val + decoded.imm) as memory::Address128;
        
        let value = cpu.get_memory().borrow_mut().read_32(addr) as u128;
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, value as i128);
        result
    }

    /// FLD: Load Double-Precision Floating-Point (加载双精度浮点数)
    ///
    /// fd = MEM[rs1 + imm][63:0]
    fn execute_fld(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1);
        let addr = (rs1_val + decoded.imm) as memory::Address128;
        
        let value = cpu.get_memory().borrow_mut().read_64(addr) as u128;
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, value as i128);
        result
    }

    /// FLQ: Load Quad-Precision Floating-Point (加载四精度浮点数)
    ///
    /// fd = MEM[rs1 + imm][127:0]
    fn execute_flq(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1);
        let addr = (rs1_val + decoded.imm) as memory::Address128;
        
        let value = cpu.get_memory().borrow_mut().read_128(addr);
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, value as i128);
        result
    }

    /// FSW: Store Single-Precision Floating-Point (存储单精度浮点数)
    ///
    /// MEM[rs1 + imm][31:0] = fs2
    fn execute_fsw(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1);
        let addr = (rs1_val + decoded.imm) as memory::Address128;
        
        let fs2_val = Self::get_reg_value(regs, decoded.rs2) as u32;
        cpu.get_memory().borrow_mut().write_32(addr, fs2_val);
        result
    }

    /// FSD: Store Double-Precision Floating-Point (存储双精度浮点数)
    ///
    /// MEM[rs1 + imm][63:0] = fs2
    fn execute_fsd(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1);
        let addr = (rs1_val + decoded.imm) as memory::Address128;
        
        let fs2_val = Self::get_reg_value(regs, decoded.rs2) as u64;
        cpu.get_memory().borrow_mut().write_64(addr, fs2_val);
        result
    }

    /// FSQ: Store Quad-Precision Floating-Point (存储四精度浮点数)
    ///
    /// MEM[rs1 + imm][127:0] = fs2
    fn execute_fsq(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1);
        let addr = (rs1_val + decoded.imm) as memory::Address128;
        
        let fs2_val = Self::get_reg_value(regs, decoded.rs2) as u128;
        cpu.get_memory().borrow_mut().write_128(addr, fs2_val);
        result
    }

    /// FADD.S/D/Q: Floating-Point Add (浮点加法)
    ///
    /// fd = fs1 + fs2
    fn execute_fp_add(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction, fmt: u8) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers_mut();
        
        match fmt {
            0 => {
                // FADD.S (single precision)
                let fs1 = Self::get_fp_reg_f32(regs, decoded.rs1);
                let fs2 = Self::get_fp_reg_f32(regs, decoded.rs2);
                let fd = fs1 + fs2;
                Self::set_fp_reg_f32(regs, decoded.rd, fd);
            }
            1 => {
                // FADD.D (double precision)
                let fs1 = Self::get_fp_reg_f64(regs, decoded.rs1);
                let fs2 = Self::get_fp_reg_f64(regs, decoded.rs2);
                let fd = fs1 + fs2;
                Self::set_fp_reg_f64(regs, decoded.rd, fd);
            }
            3 => {
                // FADD.Q (quad precision) - use f64 as approximation
                let fs1 = Self::get_fp_reg_f64(regs, decoded.rs1);
                let fs2 = Self::get_fp_reg_f64(regs, decoded.rs2);
                let fd = fs1 + fs2;
                Self::set_fp_reg_f64(regs, decoded.rd, fd);
            }
            _ => {}
        }
        result
    }

    /// FSUB.S/D/Q: Floating-Point Subtract (浮点减法)
    ///
    /// fd = fs1 - fs2
    fn execute_fp_sub(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction, fmt: u8) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers_mut();
        
        match fmt {
            0 => {
                // FSUB.S
                let fs1 = Self::get_fp_reg_f32(regs, decoded.rs1);
                let fs2 = Self::get_fp_reg_f32(regs, decoded.rs2);
                let fd = fs1 - fs2;
                Self::set_fp_reg_f32(regs, decoded.rd, fd);
            }
            1 => {
                // FSUB.D
                let fs1 = Self::get_fp_reg_f64(regs, decoded.rs1);
                let fs2 = Self::get_fp_reg_f64(regs, decoded.rs2);
                let fd = fs1 - fs2;
                Self::set_fp_reg_f64(regs, decoded.rd, fd);
            }
            3 => {
                // FSUB.Q
                let fs1 = Self::get_fp_reg_f64(regs, decoded.rs1);
                let fs2 = Self::get_fp_reg_f64(regs, decoded.rs2);
                let fd = fs1 - fs2;
                Self::set_fp_reg_f64(regs, decoded.rd, fd);
            }
            _ => {}
        }
        result
    }

    /// FMUL.S/D/Q: Floating-Point Multiply (浮点乘法)
    ///
    /// fd = fs1 * fs2
    fn execute_fp_mul(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction, fmt: u8) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers_mut();
        
        match fmt {
            0 => {
                // FMUL.S
                let fs1 = Self::get_fp_reg_f32(regs, decoded.rs1);
                let fs2 = Self::get_fp_reg_f32(regs, decoded.rs2);
                let fd = fs1 * fs2;
                Self::set_fp_reg_f32(regs, decoded.rd, fd);
            }
            1 => {
                // FMUL.D
                let fs1 = Self::get_fp_reg_f64(regs, decoded.rs1);
                let fs2 = Self::get_fp_reg_f64(regs, decoded.rs2);
                let fd = fs1 * fs2;
                Self::set_fp_reg_f64(regs, decoded.rd, fd);
            }
            3 => {
                // FMUL.Q
                let fs1 = Self::get_fp_reg_f64(regs, decoded.rs1);
                let fs2 = Self::get_fp_reg_f64(regs, decoded.rs2);
                let fd = fs1 * fs2;
                Self::set_fp_reg_f64(regs, decoded.rd, fd);
            }
            _ => {}
        }
        result
    }

    /// FDIV.S/D/Q: Floating-Point Divide (浮点除法)
    ///
    /// fd = fs1 / fs2
    fn execute_fp_div(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction, fmt: u8) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers_mut();
        
        match fmt {
            0 => {
                // FDIV.S
                let fs1 = Self::get_fp_reg_f32(regs, decoded.rs1);
                let fs2 = Self::get_fp_reg_f32(regs, decoded.rs2);
                let fd = fs1 / fs2;
                Self::set_fp_reg_f32(regs, decoded.rd, fd);
            }
            1 => {
                // FDIV.D
                let fs1 = Self::get_fp_reg_f64(regs, decoded.rs1);
                let fs2 = Self::get_fp_reg_f64(regs, decoded.rs2);
                let fd = fs1 / fs2;
                Self::set_fp_reg_f64(regs, decoded.rd, fd);
            }
            3 => {
                // FDIV.Q
                let fs1 = Self::get_fp_reg_f64(regs, decoded.rs1);
                let fs2 = Self::get_fp_reg_f64(regs, decoded.rs2);
                let fd = fs1 / fs2;
                Self::set_fp_reg_f64(regs, decoded.rd, fd);
            }
            _ => {}
        }
        result
    }

    /// FSQRT.S/D/Q: Floating-Point Square Root (浮点平方根)
    ///
    /// fd = sqrt(fs1)
    fn execute_fp_sqrt(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction, fmt: u8) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers_mut();
        
        match fmt {
            0 => {
                // FSQRT.S
                let fs1 = Self::get_fp_reg_f32(regs, decoded.rs1);
                let fd = fs1.sqrt();
                Self::set_fp_reg_f32(regs, decoded.rd, fd);
            }
            1 => {
                // FSQRT.D
                let fs1 = Self::get_fp_reg_f64(regs, decoded.rs1);
                let fd = fs1.sqrt();
                Self::set_fp_reg_f64(regs, decoded.rd, fd);
            }
            3 => {
                // FSQRT.Q
                let fs1 = Self::get_fp_reg_f64(regs, decoded.rs1);
                let fd = fs1.sqrt();
                Self::set_fp_reg_f64(regs, decoded.rd, fd);
            }
            _ => {}
        }
        result
    }

    /// FSGNJ.S/D/Q: Floating-Point Sign Injection (浮点符号注入)
    ///
    /// FSGNJ: fd = |fs1| * sign(fs2)
    /// FSGNJN: fd = |fs1| * ~sign(fs2)
    /// FSGNJX: fd = |fs1| * (sign(fs1) XOR sign(fs2))
    fn execute_fp_sgnj(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction, fmt: u8) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers_mut();
        
        match fmt {
            0 => {
                // FSGNJ.S/D/Q variants
                let fs1_bits = Self::get_reg_value(regs, decoded.rs1) as u32;
                let fs2_bits = Self::get_reg_value(regs, decoded.rs2) as u32;
                let fd_bits = match decoded.funct3 {
                    0x0 => (fs1_bits & 0x7fffffff) | (fs2_bits & 0x80000000), // FSGNJ
                    0x1 => (fs1_bits & 0x7fffffff) | (!fs2_bits & 0x80000000), // FSGNJN
                    0x2 => fs1_bits ^ (fs2_bits & 0x80000000), // FSGNJX
                    _ => fs1_bits,
                };
                Self::set_reg_value(regs, decoded.rd, fd_bits as i128);
            }
            1 => {
                // FSGNJ.D variants
                let fs1_bits = Self::get_reg_value(regs, decoded.rs1) as u64;
                let fs2_bits = Self::get_reg_value(regs, decoded.rs2) as u64;
                let fd_bits = match decoded.funct3 {
                    0x0 => (fs1_bits & 0x7fffffffffffffff) | (fs2_bits & 0x8000000000000000), // FSGNJ
                    0x1 => (fs1_bits & 0x7fffffffffffffff) | (!fs2_bits & 0x8000000000000000), // FSGNJN
                    0x2 => fs1_bits ^ (fs2_bits & 0x8000000000000000), // FSGNJX
                    _ => fs1_bits,
                };
                Self::set_reg_value(regs, decoded.rd, fd_bits as i128);
            }
            3 => {
                // FSGNJ.Q variants
                let fs1_bits = Self::get_reg_value(regs, decoded.rs1) as u128;
                let fs2_bits = Self::get_reg_value(regs, decoded.rs2) as u128;
                let fd_bits = match decoded.funct3 {
                    0x0 => (fs1_bits & 0x7fffffffffffffffffffffffffffffff) | (fs2_bits & 0x80000000000000000000000000000000), // FSGNJ
                    0x1 => (fs1_bits & 0x7fffffffffffffffffffffffffffffff) | (!fs2_bits & 0x80000000000000000000000000000000), // FSGNJN
                    0x2 => fs1_bits ^ (fs2_bits & 0x80000000000000000000000000000000), // FSGNJX
                    _ => fs1_bits,
                };
                Self::set_reg_value(regs, decoded.rd, fd_bits as i128);
            }
            _ => {}
        }
        result
    }

    /// FMIN/FMAX.S/D/Q: Floating-Point Minimum/Maximum (浮点最小/最大值)
    ///
    /// FMIN: fd = min(fs1, fs2)
    /// FMAX: fd = max(fs1, fs2)
    fn execute_fp_minmax(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction, fmt: u8) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers_mut();
        
        match fmt {
            0 => {
                // FMIN.S / FMAX.S
                let fs1 = Self::get_fp_reg_f32(regs, decoded.rs1);
                let fs2 = Self::get_fp_reg_f32(regs, decoded.rs2);
                let fd = match decoded.funct3 {
                    0x0 => fs1.min(fs2), // FMIN
                    0x1 => fs1.max(fs2), // FMAX
                    _ => fs1,
                };
                Self::set_fp_reg_f32(regs, decoded.rd, fd);
            }
            1 => {
                // FMIN.D / FMAX.D
                let fs1 = Self::get_fp_reg_f64(regs, decoded.rs1);
                let fs2 = Self::get_fp_reg_f64(regs, decoded.rs2);
                let fd = match decoded.funct3 {
                    0x0 => fs1.min(fs2), // FMIN
                    0x1 => fs1.max(fs2), // FMAX
                    _ => fs1,
                };
                Self::set_fp_reg_f64(regs, decoded.rd, fd);
            }
            3 => {
                // FMIN.Q / FMAX.Q
                let fs1 = Self::get_fp_reg_f64(regs, decoded.rs1);
                let fs2 = Self::get_fp_reg_f64(regs, decoded.rs2);
                let fd = match decoded.funct3 {
                    0x0 => fs1.min(fs2), // FMIN
                    0x1 => fs1.max(fs2), // FMAX
                    _ => fs1,
                };
                Self::set_fp_reg_f64(regs, decoded.rd, fd);
            }
            _ => {}
        }
        result
    }

    /// FLE/FLT/FEQ.S/D/Q: Floating-Point Compare (浮点比较)
    ///
    /// FLE: rd = (fs1 <= fs2) ? 1 : 0
    /// FLT: rd = (fs1 < fs2) ? 1 : 0
    /// FEQ: rd = (fs1 == fs2) ? 1 : 0
    fn execute_fp_cmp(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction, fmt: u8) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers();
        
        let cmp_result = match fmt {
            0 => {
                // FLE.S / FLT.S / FEQ.S
                let fs1 = Self::get_fp_reg_f32(regs, decoded.rs1);
                let fs2 = Self::get_fp_reg_f32(regs, decoded.rs2);
                match decoded.funct3 {
                    0x0 => (fs1 <= fs2) as i128, // FLE
                    0x1 => (fs1 < fs2) as i128,  // FLT
                    0x2 => (fs1 == fs2) as i128, // FEQ
                    _ => 0,
                }
            }
            1 => {
                // FLE.D / FLT.D / FEQ.D
                let fs1 = Self::get_fp_reg_f64(regs, decoded.rs1);
                let fs2 = Self::get_fp_reg_f64(regs, decoded.rs2);
                match decoded.funct3 {
                    0x0 => (fs1 <= fs2) as i128, // FLE
                    0x1 => (fs1 < fs2) as i128,  // FLT
                    0x2 => (fs1 == fs2) as i128, // FEQ
                    _ => 0,
                }
            }
            3 => {
                // FLE.Q / FLT.Q / FEQ.Q
                let fs1 = Self::get_fp_reg_f64(regs, decoded.rs1);
                let fs2 = Self::get_fp_reg_f64(regs, decoded.rs2);
                match decoded.funct3 {
                    0x0 => (fs1 <= fs2) as i128, // FLE
                    0x1 => (fs1 < fs2) as i128,  // FLT
                    0x2 => (fs1 == fs2) as i128, // FEQ
                    _ => 0,
                }
            }
            _ => 0,
        };
        
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, cmp_result);
        result
    }

    /// FCVT.W/L.S/D/Q: Floating-Point Convert to/from Integer (浮点到整数转换)
    ///
    /// Various conversion instructions between floating-point and integer formats.
    fn execute_fp_cvt_int(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction, _fmt: u8) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers();
        
        // rs2 encodes the conversion type
        match decoded.rs2 {
            // FCVT.W.S: Convert single to signed 32-bit
            0x00 => {
                let fs1 = Self::get_fp_reg_f32(regs, decoded.rs1);
                let val = fs1 as i32 as i128;
                Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, val);
            }
            // FCVT.W.D: Convert double to signed 32-bit
            0x01 => {
                let fs1 = Self::get_fp_reg_f64(regs, decoded.rs1);
                let val = fs1 as i32 as i128;
                Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, val);
            }
            // FCVT.W.Q: Convert quad to signed 32-bit
            0x03 => {
                let fs1 = Self::get_fp_reg_f64(regs, decoded.rs1);
                let val = fs1 as i32 as i128;
                Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, val);
            }
            // FCVT.WU.S: Convert single to unsigned 32-bit
            0x04 => {
                let fs1 = Self::get_fp_reg_f32(regs, decoded.rs1);
                let val = fs1 as u32 as i128;
                Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, val);
            }
            // FCVT.WU.D: Convert double to unsigned 32-bit
            0x05 => {
                let fs1 = Self::get_fp_reg_f64(regs, decoded.rs1);
                let val = fs1 as u32 as i128;
                Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, val);
            }
            // FCVT.WU.Q: Convert quad to unsigned 32-bit
            0x07 => {
                let fs1 = Self::get_fp_reg_f64(regs, decoded.rs1);
                let val = fs1 as u32 as i128;
                Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, val);
            }
            // FCVT.L.S: Convert single to signed 64-bit
            0x08 => {
                let fs1 = Self::get_fp_reg_f32(regs, decoded.rs1);
                let val = fs1 as i64 as i128;
                Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, val);
            }
            // FCVT.L.D: Convert double to signed 64-bit
            0x09 => {
                let fs1 = Self::get_fp_reg_f64(regs, decoded.rs1);
                let val = fs1 as i64 as i128;
                Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, val);
            }
            // FCVT.L.Q: Convert quad to signed 64-bit
            0x0b => {
                let fs1 = Self::get_fp_reg_f64(regs, decoded.rs1);
                let val = fs1 as i64 as i128;
                Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, val);
            }
            // FCVT.LU.S: Convert single to unsigned 64-bit
            0x0c => {
                let fs1 = Self::get_fp_reg_f32(regs, decoded.rs1);
                let val = fs1 as u64 as i128;
                Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, val);
            }
            // FCVT.LU.D: Convert double to unsigned 64-bit
            0x0d => {
                let fs1 = Self::get_fp_reg_f64(regs, decoded.rs1);
                let val = fs1 as u64 as i128;
                Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, val);
            }
            // FCVT.LU.Q: Convert quad to unsigned 64-bit
            0x0f => {
                let fs1 = Self::get_fp_reg_f64(regs, decoded.rs1);
                let val = fs1 as u64 as i128;
                Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, val);
            }
            // FCVT.S.W: Convert signed 32-bit to single
            0x10 => {
                let rs1_val = Self::get_reg_value(regs, decoded.rs1) as i32;
                let fd = rs1_val as f32;
                Self::set_fp_reg_f32(cpu.get_registers_mut(), decoded.rd, fd);
            }
            // FCVT.D.W: Convert signed 32-bit to double
            0x11 => {
                let rs1_val = Self::get_reg_value(regs, decoded.rs1) as i32;
                let fd = rs1_val as f64;
                Self::set_fp_reg_f64(cpu.get_registers_mut(), decoded.rd, fd);
            }
            // FCVT.Q.W: Convert signed 32-bit to quad
            0x13 => {
                let rs1_val = Self::get_reg_value(regs, decoded.rs1) as i32;
                let fd = rs1_val as f64;
                Self::set_fp_reg_f64(cpu.get_registers_mut(), decoded.rd, fd);
            }
            // FCVT.S.WU: Convert unsigned 32-bit to single
            0x14 => {
                let rs1_val = Self::get_reg_value(regs, decoded.rs1) as u32;
                let fd = rs1_val as f32;
                Self::set_fp_reg_f32(cpu.get_registers_mut(), decoded.rd, fd);
            }
            // FCVT.D.WU: Convert unsigned 32-bit to double
            0x15 => {
                let rs1_val = Self::get_reg_value(regs, decoded.rs1) as u32;
                let fd = rs1_val as f64;
                Self::set_fp_reg_f64(cpu.get_registers_mut(), decoded.rd, fd);
            }
            // FCVT.Q.WU: Convert unsigned 32-bit to quad
            0x17 => {
                let rs1_val = Self::get_reg_value(regs, decoded.rs1) as u32;
                let fd = rs1_val as f64;
                Self::set_fp_reg_f64(cpu.get_registers_mut(), decoded.rd, fd);
            }
            // FCVT.S.L: Convert signed 64-bit to single
            0x18 => {
                let rs1_val = Self::get_reg_value(regs, decoded.rs1) as i64;
                let fd = rs1_val as f32;
                Self::set_fp_reg_f32(cpu.get_registers_mut(), decoded.rd, fd);
            }
            // FCVT.D.L: Convert signed 64-bit to double
            0x19 => {
                let rs1_val = Self::get_reg_value(regs, decoded.rs1) as i64;
                let fd = rs1_val as f64;
                Self::set_fp_reg_f64(cpu.get_registers_mut(), decoded.rd, fd);
            }
            // FCVT.Q.L: Convert signed 64-bit to quad
            0x1b => {
                let rs1_val = Self::get_reg_value(regs, decoded.rs1) as i64;
                let fd = rs1_val as f64;
                Self::set_fp_reg_f64(cpu.get_registers_mut(), decoded.rd, fd);
            }
            // FCVT.S.LU: Convert unsigned 64-bit to single
            0x1c => {
                let rs1_val = Self::get_reg_value(regs, decoded.rs1) as u64;
                let fd = rs1_val as f32;
                Self::set_fp_reg_f32(cpu.get_registers_mut(), decoded.rd, fd);
            }
            // FCVT.D.LU: Convert unsigned 64-bit to double
            0x1d => {
                let rs1_val = Self::get_reg_value(regs, decoded.rs1) as u64;
                let fd = rs1_val as f64;
                Self::set_fp_reg_f64(cpu.get_registers_mut(), decoded.rd, fd);
            }
            // FCVT.Q.LU: Convert unsigned 64-bit to quad
            0x1f => {
                let rs1_val = Self::get_reg_value(regs, decoded.rs1) as u64;
                let fd = rs1_val as f64;
                Self::set_fp_reg_f64(cpu.get_registers_mut(), decoded.rd, fd);
            }
            _ => {}
        }
        result
    }

    /// FMV.X.S/D/Q, FCLASS.S/D/Q: Move and Classify (移动和分类)
    ///
    /// FMV.X.S/D/Q: rd = f[rs1] (bitwise move)
    /// FCLASS.S/D/Q: rd = classify(f[rs1])
    fn execute_fp_mvf_class(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction, fmt: u8) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers();
        
        if decoded.rs2 == 0 {
            // FMV.X.S/D/Q: Bitwise move from FP register to integer register
            let fp_val = Self::get_reg_value(regs, decoded.rs1);
            Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, fp_val);
        } else if decoded.rs2 == 1 {
            // FCLASS.S/D/Q: Classify floating-point number
            let class = match fmt {
                0 => {
                    // FCLASS.S
                    let fs1 = Self::get_fp_reg_f32(regs, decoded.rs1);
                    Self::classify_f32(fs1)
                }
                1 => {
                    // FCLASS.D
                    let fs1 = Self::get_fp_reg_f64(regs, decoded.rs1);
                    Self::classify_f64(fs1)
                }
                3 => {
                    // FCLASS.Q
                    let fs1 = Self::get_fp_reg_f64(regs, decoded.rs1);
                    Self::classify_f64(fs1)
                }
                _ => 0,
            };
            Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, class);
        }
        result
    }

    /// Helper: Classify f32 number for FCLASS instruction.
    ///
    /// Returns a 10-bit mask indicating the class of the number.
    fn classify_f32(f: f32) -> i128 {
        if f.is_nan() {
            if f32::is_sign_positive(f) && f.is_nan() {
                return 1 << 9; // Quiet NaN (positive)
            } else if f.is_nan() {
                return 1 << 8; // Quiet NaN (negative) - simplified
            }
            return 1 << 9; // Quiet NaN
        }
        if f == 0.0 {
            return 1 << (if f32::is_sign_positive(f) { 4 } else { 3 }); // Zero
        }
        if f.is_infinite() {
            return 1 << (if f32::is_sign_positive(f) { 7 } else { 0 }); // Infinity
        }
        if f.is_subnormal() {
            return 1 << (if f32::is_sign_positive(f) { 6 } else { 1 }); // Subnormal
        }
        // Normal number
        1 << (if f32::is_sign_positive(f) { 5 } else { 2 })
    }

    /// Helper: Classify f64 number for FCLASS instruction.
    ///
    /// Returns a 10-bit mask indicating the class of the number.
    fn classify_f64(f: f64) -> i128 {
        if f.is_nan() {
            return 1 << 9; // Quiet NaN
        }
        if f == 0.0 {
            return 1 << (if f64::is_sign_positive(f) { 4 } else { 3 }); // Zero
        }
        if f.is_infinite() {
            return 1 << (if f64::is_sign_positive(f) { 7 } else { 0 }); // Infinity
        }
        if f.is_subnormal() {
            return 1 << (if f64::is_sign_positive(f) { 6 } else { 1 }); // Subnormal
        }
        // Normal number
        1 << (if f64::is_sign_positive(f) { 5 } else { 2 })
    }

    /// FMV.S/D/Q.X: Move from Integer to Floating-Point (从整数移动到浮点)
    ///
    /// fd = x[rs1] (bitwise move)
    fn execute_fp_mvt(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction, _fmt: u8) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers();
        
        let rs1_val = Self::get_reg_value(regs, decoded.rs1);
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, rs1_val);
        result
    }

    // ========================================
    // V Extension: Vector Instructions / V 扩展：向量指令
    // ========================================

    /// Helper: Get vector register value.
    fn get_v_reg_value(regs: &register::Register, reg: u8) -> u128 {
        let v_index = unsafe { std::mem::transmute::<u8, register::VRegisterIndex>(reg) };
        regs.read_v(v_index)
    }

    /// Helper: Set vector register value.
    fn set_v_reg_value(regs: &mut register::Register, reg: u8, value: u128) {
        let v_index = unsafe { std::mem::transmute::<u8, register::VRegisterIndex>(reg) };
        regs.write_v(v_index, value);
    }

    /// Execute Vector-Vector Integer operations (OPIVV).
    fn execute_vector_opivv(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction, funct6: u8) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers();
        let vs1 = Self::get_v_reg_value(regs, decoded.rs1);
        let vs2 = Self::get_v_reg_value(regs, decoded.rs2);
        let vd = match funct6 {
            0x00 => vs2.wrapping_add(vs1),   // VADD.VV
            0x02 => vs2.wrapping_sub(vs1),   // VSUB.VV
            0x04 => (vs2 as i128).min(vs1 as i128) as u128, // VMIN.VV
            0x05 => vs2.min(vs1),            // VMINU.VV
            0x06 => (vs2 as i128).max(vs1 as i128) as u128, // VMAX.VV
            0x07 => vs2.max(vs1),            // VMAXU.VV
            0x08 => vs2 & vs1,               // VAND.VV
            0x09 => vs2 | vs1,               // VOR.VV
            0x0a => vs2 ^ vs1,               // VXOR.VV
            0x0c => vs2 << (vs1 & 0x7f),     // VSLL.VV
            0x0d => vs2 >> (vs1 & 0x7f),     // VSRL.VV
            0x0e => ((vs2 as i128) >> (vs1 & 0x7f)) as u128, // VSRA.VV
            0x18 => vs2,                     // VMV.V.V
            _ => 0,
        };
        Self::set_v_reg_value(cpu.get_registers_mut(), decoded.rd, vd);
        result
    }

    /// Execute Vector-Vector Floating-Point operations (OPFVV).
    fn execute_vector_opfvv(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction, funct6: u8) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers();
        let vs1_f64 = f64::from_bits(Self::get_v_reg_value(regs, decoded.rs1) as u64);
        let vs2_f64 = f64::from_bits(Self::get_v_reg_value(regs, decoded.rs2) as u64);
        
        let vd = match funct6 {
            0x00 => (vs2_f64 + vs1_f64).to_bits() as u128, // VFADD.VV
            0x02 => (vs2_f64 - vs1_f64).to_bits() as u128, // VFSUB.VV
            0x04 => vs2_f64.min(vs1_f64).to_bits() as u128, // VFMIN.VV
            0x05 => vs2_f64.max(vs1_f64).to_bits() as u128, // VFMAX.VV
            0x18 => vs2_f64.to_bits() as u128,            // VFMV.V.F
            _ => 0,
        };
        Self::set_v_reg_value(cpu.get_registers_mut(), decoded.rd, vd);
        result
    }

    /// Execute Vector Multiply/Divide operations (OPMVV).
    fn execute_vector_opmvv(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction, funct6: u8) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers();
        let vs1 = Self::get_v_reg_value(regs, decoded.rs1);
        let vs2 = Self::get_v_reg_value(regs, decoded.rs2);
        
        let vd = match funct6 {
            0x0d => if vs1 != 0 { vs2.wrapping_div(vs1) } else { u128::MAX }, // VDIV.VV
            0x0f => if vs1 != 0 { vs2.wrapping_rem(vs1) } else { vs2 },       // VREM.VV
            0x11 => vs2.wrapping_mul(vs1),       // VMUL.VV
            0x13 => ((vs2 as i128).wrapping_mul(vs1 as i128)) as u128, // VMULH.VV
            0x18 => vs2.wrapping_mul(vs1),       // VMACC.VV (simplified)
            _ => 0,
        };
        Self::set_v_reg_value(cpu.get_registers_mut(), decoded.rd, vd);
        result
    }

    /// Execute Vector-Immediate integer operations (OPVI).
    fn execute_vector_opvi(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction, funct6: u8) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers();
        let vs2 = Self::get_v_reg_value(regs, decoded.rs2);
        let rs1 = Self::get_reg_value(regs, decoded.rs1) as u128;
        
        let vd = match funct6 {
            0x00 => vs2.wrapping_add(rs1),       // VADD.VX
            0x02 => vs2.wrapping_sub(rs1),       // VSUB.VX
            0x04 => (vs2 as i128).min(rs1 as i128) as u128, // VMIN.VX
            0x05 => vs2.min(rs1),                // VMINU.VX
            0x06 => (vs2 as i128).max(rs1 as i128) as u128, // VMAX.VX
            0x07 => vs2.max(rs1),                // VMAXU.VX
            0x08 => vs2 & rs1,                   // VAND.VX
            0x09 => vs2 | rs1,                   // VOR.VX
            0x0a => vs2 ^ rs1,                   // VXOR.VX
            0x0c => vs2 << (rs1 & 0x7f),         // VSLL.VX
            0x0d => vs2 >> (rs1 & 0x7f),         // VSRL.VX
            0x0e => ((vs2 as i128) >> (rs1 & 0x7f)) as u128, // VSRA.VX
            _ => 0,
        };
        Self::set_v_reg_value(cpu.get_registers_mut(), decoded.rd, vd);
        result
    }

    /// Execute Vector-5-bit immediate operations (OPIVI).
    fn execute_vector_opivi(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction, funct6: u8) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers();
        let vs2 = Self::get_v_reg_value(regs, decoded.rs2);
        // 5-bit immediate is sign-extended
        let imm = ((decoded.rs1 as i8) << 3 >> 3) as i128 as u128;
        
        let vd = match funct6 {
            0x00 => vs2.wrapping_add(imm),       // VADD.VI
            0x08 => vs2 & imm,                   // VAND.VI
            0x09 => vs2 | imm,                   // VOR.VI
            0x0a => vs2 ^ imm,                   // VXOR.VI
            0x0c => vs2 << (imm & 0x7f),         // VSLL.VI
            0x0d => vs2 >> (imm & 0x7f),         // VSRL.VI
            0x0e => ((vs2 as i128) >> (imm & 0x7f)) as u128, // VSRA.VI
            _ => 0,
        };
        Self::set_v_reg_value(cpu.get_registers_mut(), decoded.rd, vd);
        result
    }

    /// Execute Vector-Scalar FP operations (OPFVF).
    fn execute_vector_opfvf(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction, funct6: u8) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers();
        let vs2_f64 = f64::from_bits(Self::get_v_reg_value(regs, decoded.rs2) as u64);
        let rs1_f64 = f64::from_bits(Self::get_reg_value(regs, decoded.rs1) as u64);
        
        let vd = match funct6 {
            0x00 => (vs2_f64 + rs1_f64).to_bits() as u128, // VFADD.VF
            0x02 => (vs2_f64 - rs1_f64).to_bits() as u128, // VFSUB.VF
            0x04 => vs2_f64.min(rs1_f64).to_bits() as u128, // VFMIN.VF
            0x05 => vs2_f64.max(rs1_f64).to_bits() as u128, // VFMAX.VF
            0x18 => rs1_f64.to_bits() as u128,             // VFMV.V.F
            _ => 0,
        };
        Self::set_v_reg_value(cpu.get_registers_mut(), decoded.rd, vd);
        result
    }

    /// Execute Vector Load operations.
    fn execute_vector_load(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction, _funct6: u8) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers();
        let rs1 = Self::get_reg_value(regs, decoded.rs1);
        let addr = rs1 as memory::Address128;
        
        // For simplicity, load a single 128-bit value
        // In a full implementation, this would load multiple elements based on VL
        let vd = cpu.get_memory().borrow_mut().read_128(addr);
        Self::set_v_reg_value(cpu.get_registers_mut(), decoded.rd, vd);
        result
    }

    /// Execute Vector Configuration instructions.
    fn execute_vector_config(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction, _funct6: u8) -> ExecutionResult {
        let result = ExecutionResult::new();
        let funct6 = (decoded.funct7 >> 1) & 0x3f;
        
        match funct6 {
            0x30 => {
                // VSETVLI: Set vector length
                let rs1_val = Self::get_reg_value(cpu.get_registers(), decoded.rs1) as u64;
                let zimm = decoded.rs2 as u64;
                
                // Extract vtype from zimm
                let vtype = register::VType::from_u64(zimm);
                let vector_state = cpu.get_registers_mut().get_vector_state_mut();
                vector_state.vtype = vtype;
                
                // Set VL
                let requested_vl = if decoded.rs1 == 0 {
                    vector_state.vlmax()
                } else {
                    rs1_val
                };
                vector_state.set_vl(requested_vl);
                let vl = vector_state.vl;
                
                // Write VL to rd
                Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, vl as i128);
            }
            0x31 => {
                // VSETIVLI: Set vector length with immediate
                let zimm = decoded.rs2 as u64;
                let vtype = register::VType::from_u64(zimm);
                let avl = decoded.rs1 as u64;
                
                let vector_state = cpu.get_registers_mut().get_vector_state_mut();
                vector_state.vtype = vtype;
                vector_state.set_vl(avl);
                let vl = vector_state.vl;
                
                Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, vl as i128);
            }
            0x3f => {
                // VSETVL: Set vector length from register
                let rs1_val = Self::get_reg_value(cpu.get_registers(), decoded.rs1) as u64;
                let rs2_val = Self::get_reg_value(cpu.get_registers(), decoded.rs2) as u64;
                
                let vtype = register::VType::from_u64(rs2_val);
                let vector_state = cpu.get_registers_mut().get_vector_state_mut();
                vector_state.vtype = vtype;
                vector_state.set_vl(rs1_val);
                let vl = vector_state.vl;
                
                Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, vl as i128);
            }
            _ => {}
        }
        
        result
    }
}
