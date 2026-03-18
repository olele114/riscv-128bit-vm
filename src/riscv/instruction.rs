//! Instruction Decoder Module
//!
//! Provides instruction decoding functionality for RISC-V instructions.
//! Parses 32-bit instruction words and extracts opcode, registers,
//! function codes, and immediate values.
//!
//! # Supported Instruction Formats
//!
//! - R-type: Register-register operations (ADD, SUB, SLL, etc.)
//! - I-type: Register-immediate operations (ADDI, LOAD, JALR, etc.)
//! - S-type: Store operations (SB, SH, SW, SD, SQ)
//! - B-type: Branch operations (BEQ, BNE, BLT, etc.)
//! - U-type: Upper immediate operations (LUI, AUIPC)
//! - J-type: Jump operations (JAL)
//!
//! ---
//!
//! 指令解码器模块
//!
//! 提供 RISC-V 指令的解码功能。
//! 解析 32 位指令字并提取操作码、寄存器、功能码和立即数。
//!
//! # 支持的指令格式
//!
//! - R-type: 寄存器-寄存器操作 (ADD, SUB, SLL 等)
//! - I-type: 寄存器-立即数操作 (ADDI, LOAD, JALR 等)
//! - S-type: 存储操作 (SB, SH, SW, SD, SQ)
//! - B-type: 分支操作 (BEQ, BNE, BLT 等)
//! - U-type: 上位立即数操作 (LUI, AUIPC)
//! - J-type: 跳转操作 (JAL)

#![allow(dead_code)]

/// RISC-V operation codes.
///
/// The opcode determines the basic operation type.
/// Combined with funct3 and funct7, it fully specifies the instruction.
///
/// ---
///
/// RISC-V 操作码。
///
/// 操作码决定基本操作类型。
/// 与 funct3 和 funct7 组合可完全指定指令。
#[repr(u8)]
pub enum OpCode {
    /// Load Upper Immediate (加载上位立即数)
    Lui = 0x37,
    /// Add Upper Immediate to PC (将上位立即数加到 PC)
    Auipc = 0x17,
    /// Jump and Link (跳转并链接)
    Jal = 0x6f,
    /// Jump and Link Register (寄存器跳转并链接)
    Jalr = 0x67,
    /// Branch (分支)
    Branch = 0x63,
    /// Load (加载)
    Load = 0x03,
    /// Store (存储)
    Store = 0x23,
    /// Immediate arithmetic (立即数算术)
    Imm = 0x13,
    /// Register arithmetic (寄存器算术)
    Reg = 0x33,
    /// System instructions (系统指令)
    System = 0x73,
    /// Miscellaneous memory (杂项内存)
    MiscMem = 0x0f,
}

/// Function 3 codes for instruction variants.
///
/// ---
///
/// 指令变体的 funct3 代码。
#[repr(u8)]
enum Funct3 {
    AddSub = 0x0,
    Sll = 0x1,
    Slt = 0x2,
    Sltu = 0x3,
    Xor = 0x4,
    SrSrlSra = 0x5,
    Or = 0x6,
    And = 0x7,
}

/// Function 7 codes for instruction variants.
///
/// ---
///
/// 指令变体的 funct7 代码。
#[repr(u8)]
enum Funct7 {
    None = 0x00,
    SubSra = 0x20,
    MulDiv = 0x01,
}

/// Branch function 3 codes.
///
/// Determines the comparison type for branch instructions.
///
/// ---
///
/// 分支 funct3 代码。
///
/// 决定分支指令的比较类型。
#[repr(u8)]
pub(crate) enum BranchFunct3 {
    /// Branch if equal (相等则分支)
    Beq = 0x0,
    /// Branch if not equal (不等则分支)
    Bne = 0x1,
    /// Branch if less than (小于则分支)
    Blt = 0x4,
    /// Branch if greater or equal (大于等于则分支)
    Bge = 0x5,
    /// Branch if less than unsigned (无符号小于则分支)
    Bltu = 0x6,
    /// Branch if greater or equal unsigned (无符号大于等于则分支)
    Bgeu = 0x7,
}

/// Load function 3 codes.
///
/// Determines the size and sign-extension for load instructions.
///
/// ---
///
/// 加载 funct3 代码。
///
/// 决定加载指令的大小和符号扩展。
#[repr(u8)]
enum LoadFunct3 {
    /// Load byte (加载字节，符号扩展)
    Lb = 0x0,
    /// Load halfword (加载半字，符号扩展)
    Lh = 0x1,
    /// Load word (加载字，符号扩展)
    Lw = 0x2,
    /// Load doubleword (加载双字，符号扩展)
    Ld = 0x3,
    /// Load quadword (加载四字，符号扩展)
    Lq = 0x4,
    /// Load halfword unsigned (加载半字，零扩展)
    Lhu = 0x5,
    /// Load word unsigned (加载字，零扩展)
    Lwu = 0x6,
    /// Load doubleword unsigned (加载双字，零扩展)
    Ldu = 0x7,
}

/// Store function 3 codes.
///
/// Determines the size for store instructions.
///
/// ---
///
/// 存储 funct3 代码。
///
/// 决定存储指令的大小。
#[repr(u8)]
enum StoreFunct3 {
    /// Store byte (存储字节)
    Sb = 0x0,
    /// Store halfword (存储半字)
    Sh = 0x1,
    /// Store word (存储字)
    Sw = 0x2,
    /// Store doubleword (存储双字)
    Sd = 0x3,
    /// Store quadword (存储四字)
    Sq = 0x4,
}

/// System instruction function 3 codes.
///
/// ---
///
/// 系统指令 funct3 代码。
#[repr(u8)]
enum SystemFunct3 {
    Priv = 0x0,
    Csrrw = 0x1,
    Csrrs = 0x2,
    Csrrc = 0x3,
    Csrrwi = 0x5,
    Csrrsi = 0x6,
    Csrrci = 0x7,
}

/// RISC-V instruction format types.
///
/// ---
///
/// RISC-V 指令格式类型。
pub(crate) enum InstructionType {
    /// Register-register format (寄存器-寄存器格式)
    RType,
    /// Register-immediate format (寄存器-立即数格式)
    IType,
    /// Store format (存储格式)
    SType,
    /// Branch format (分支格式)
    BType,
    /// Upper immediate format (上位立即数格式)
    UType,
    /// Jump format (跳转格式)
    JType,
    /// Unknown format (未知格式)
    Unknown,
}

/// Decoded RISC-V instruction.
///
/// Contains all fields extracted from a 32-bit instruction word:
/// opcode, destination register (rd), source registers (rs1, rs2),
/// function codes (funct3, funct7), and immediate value.
///
/// ---
///
/// 解码后的 RISC-V 指令。
///
/// 包含从 32 位指令字提取的所有字段：
/// 操作码、目标寄存器 (rd)、源寄存器 (rs1, rs2)、
/// 功能码 (funct3, funct7) 和立即数值。
pub struct DecodedInstruction {
    /// Operation code (操作码)
    pub(crate) opcode: OpCode,
    typ: InstructionType,
    /// Destination register (目标寄存器)
    pub(crate) rd: u8,
    /// Source register 1 (源寄存器 1)
    pub(crate) rs1: u8,
    /// Source register 2 (源寄存器 2)
    pub(crate) rs2: u8,
    /// Function code 3 (功能码 3)
    pub(crate) funct3: u8,
    /// Function code 7 (功能码 7)
    pub(crate) funct7: u8,
    /// Immediate value (立即数值)
    pub(crate) imm: i128,
}

/// RISC-V Instruction Decoder.
///
/// Provides static methods for decoding 32-bit RISC-V instructions.
///
/// ---
///
/// RISC-V 指令解码器。
///
/// 提供解码 32 位 RISC-V 指令的静态方法。
pub struct InstructionDecoder {}

impl DecodedInstruction {
    /// Creates a new decoded instruction with default values.
    ///
    /// ---
    ///
    /// 创建具有默认值的新解码指令。
    fn new() -> Self {
        Self {
            opcode: OpCode::System,
            typ: InstructionType::Unknown,
            rd: 0,
            rs1: 0,
            rs2: 0,
            funct3: 0,
            funct7: 0,
            imm: 0,
        }
    }
}

impl InstructionDecoder {
    /// Extracts the opcode from a 32-bit instruction.
    ///
    /// Bits \[6:0\]
    ///
    /// ---
    ///
    /// 从 32 位指令提取操作码。
    ///
    /// 位 \[6:0\]
    pub fn extract_opcode(instruction: u32) -> u8 {
        return (instruction & 0x7f) as u8;
    }

    /// Extracts the destination register from a 32-bit instruction.
    ///
    /// Bits \[11:7\]
    ///
    /// ---
    ///
    /// 从 32 位指令提取目标寄存器。
    ///
    /// 位 \[11:7\]
    pub fn extract_rd(instruction: u32) -> u8 {
        return ((instruction >> 7) & 0x1f) as u8;
    }

    /// Extracts funct3 from a 32-bit instruction.
    ///
    /// Bits \[14:12\]
    ///
    /// ---
    ///
    /// 从 32 位指令提取 funct3。
    ///
    /// 位 \[14:12\]
    pub fn extract_funct3(instruction: u32) -> u8 {
        return ((instruction >> 12) & 0x7) as u8;
    }

    /// Extracts source register 1 from a 32-bit instruction.
    ///
    /// Bits \[19:15\]
    ///
    /// ---
    ///
    /// 从 32 位指令提取源寄存器 1。
    ///
    /// 位 \[19:15\]
    pub fn extract_rs1(instruction: u32) -> u8 {
        return ((instruction >> 15) & 0x1f) as u8;
    }

    /// Extracts source register 2 from a 32-bit instruction.
    ///
    /// Bits \[24:20\]
    ///
    /// ---
    ///
    /// 从 32 位指令提取源寄存器 2。
    ///
    /// 位 \[24:20\]
    pub fn extract_rs2(instruction: u32) -> u8 {
        return ((instruction >> 20) & 0x1f) as u8;
    }

    /// Extracts funct7 from a 32-bit instruction.
    ///
    /// Bits \[31:25\]
    ///
    /// ---
    ///
    /// 从 32 位指令提取 funct7。
    ///
    /// 位 \[31:25\]
    pub fn extract_funct7(instruction: u32) -> u8 {
        return ((instruction >> 25) & 0x7f) as u8;
    }

    /// Sign-extends a value from the specified number of bits.
    ///
    /// ---
    ///
    /// 从指定位数对值进行符号扩展。
    pub fn sign_extend(value: u64, bits: u8) -> i128 {
        let mask = 1 << (bits - 1);
        if value & mask != 0 {
            let extension = !((1 << bits) - 1);
            return (value | extension) as i128;
        }
        value as i128
    }

    /// Extracts and sign-extends I-type immediate.
    ///
    /// Bits \[31:20\]
    ///
    /// ---
    ///
    /// 提取并符号扩展 I 型立即数。
    ///
    /// 位 \[31:20\]
    pub fn extract_imm_i(instruction: u32) -> i128 {
        let imm = (instruction >> 20) & 0xfff;
        Self::sign_extend(imm as u64, 12)
    }

    /// Extracts and sign-extends S-type immediate.
    ///
    /// Bits \[31:25\] and \[11:7\]
    ///
    /// ---
    ///
    /// 提取并符号扩展 S 型立即数。
    ///
    /// 位 \[31:25\] 和 \[11:7\]
    pub fn extract_imm_s(instruction: u32) -> i128 {
        let imm = ((instruction >> 7) & 0x1f) | (((instruction >> 25) & 0x7f) << 5);
        Self::sign_extend(imm as u64, 12)
    }

    /// Extracts and sign-extends B-type immediate.
    ///
    /// Bits \[31\], \[7\], \[30:25\], \[11:8\]
    ///
    /// ---
    ///
    /// 提取并符号扩展 B 型立即数。
    ///
    /// 位 \[31\], \[7\], \[30:25\], \[11:8\]
    pub fn extract_imm_b(instruction: u32) -> i128 {
        let imm = (((instruction >> 8) & 0xf) << 1) |
            (((instruction >> 25) & 0x3f) << 5) |
            (((instruction >> 7) & 0x1) << 11) |
            (((instruction >> 31) & 0x1) << 12);
        Self::sign_extend(imm as u64, 13)
    }

    /// Extracts U-type immediate.
    ///
    /// Bits \[31:12\]
    ///
    /// ---
    ///
    /// 提取 U 型立即数。
    ///
    /// 位 \[31:12\]
    pub fn extract_imm_u(instruction: u32) -> i128 {
        (instruction & 0xfffff000) as i128
    }

    /// Extracts and sign-extends J-type immediate.
    ///
    /// Bits: \[31\], \[30:21\], \[20\], \[19:12\]
    ///
    /// ---
    ///
    /// 提取并符号扩展 J 型立即数。
    ///
    /// 位 \[31\], \[30:21\], \[20\], \[19:12\]
    pub fn extract_imm_j(instruction: u32) -> i128 {
        let imm = (((instruction >> 21) & 0x3ff) << 1) |
            (((instruction >> 20) & 0x1) << 11) |
            (((instruction >> 12) & 0xff) << 12) |
            (((instruction >> 31) & 0x1) << 20);
        Self::sign_extend(imm as u64, 21)
    }

    /// Determines the instruction type from opcode.
    ///
    /// ---
    ///
    /// 从操作码确定指令类型。
    pub fn get_instruction_type(op_code: &OpCode, _funct3: u8) -> InstructionType {
        match op_code {
            OpCode::Lui | OpCode::Auipc => InstructionType::UType,
            OpCode::Jal => InstructionType::JType,
            OpCode::Jalr | OpCode::Load | OpCode::Imm | OpCode::System => InstructionType::IType,
            OpCode::Store => InstructionType::SType,
            OpCode::Branch => InstructionType::BType,
            OpCode::Reg => InstructionType::RType,
            _ => InstructionType::Unknown,
        }
    }

    /// Decodes a 32-bit instruction into a DecodedInstruction.
    ///
    /// Extracts all fields and determines the instruction type.
    ///
    /// ---
    ///
    /// 将 32 位指令解码为 DecodedInstruction。
    ///
    /// 提取所有字段并确定指令类型。
    pub fn decode(instruction: u32) -> DecodedInstruction {
        let mut decoded = DecodedInstruction::new();

        let opcode_value = Self::extract_opcode(instruction);
        decoded.opcode = match opcode_value {
            0x37 => OpCode::Lui,
            0x17 => OpCode::Auipc,
            0x6f => OpCode::Jal,
            0x67 => OpCode::Jalr,
            0x63 => OpCode::Branch,
            0x03 => OpCode::Load,
            0x23 => OpCode::Store,
            0x13 => OpCode::Imm,
            0x33 => OpCode::Reg,
            0x73 => OpCode::System,
            0x0f => OpCode::MiscMem,
            _ => OpCode::System,
        };

        decoded.funct3 = Self::extract_funct3(instruction);
        decoded.funct7 = Self::extract_funct7(instruction);
        decoded.rd = Self::extract_rd(instruction);
        decoded.rs1 = Self::extract_rs1(instruction);
        decoded.rs2 = Self::extract_rs2(instruction);

        decoded.typ = Self::get_instruction_type(&decoded.opcode, decoded.funct3);

        match &decoded.typ {
            InstructionType::IType => {
                decoded.imm = Self::extract_imm_i(instruction);
            }
            InstructionType::SType => {
                decoded.imm = Self::extract_imm_s(instruction);
            }
            InstructionType::BType => {
                decoded.imm = Self::extract_imm_b(instruction);
            }
            InstructionType::UType => {
                decoded.imm = Self::extract_imm_u(instruction);
            }
            InstructionType::JType => {
                decoded.imm = Self::extract_imm_j(instruction);
            }
            _ => decoded.imm = 0,
        }

        decoded
    }

    /// Returns the instruction mnemonic name.
    ///
    /// ---
    ///
    /// 返回指令助记符名称。
    pub fn get_instruction_name(decoded: &DecodedInstruction) -> &'static str {
        match decoded.opcode {
            OpCode::Lui => "LUI",
            OpCode::Auipc => "AUIPC",
            OpCode::Jal => "JAL",
            OpCode::Jalr => "JALR",

            OpCode::Branch => {
                match decoded.funct3 {
                    0x0 => "BEQ",
                    0x1 => "BNE",
                    0x4 => "BLT",
                    0x5 => "BGE",
                    0x6 => "BLTU",
                    0x7 => "BGEU",
                    _ => "B_UNKNOWN",
                }
            }

            OpCode::Load => {
                match decoded.funct3 {
                    0x0 => "LB",
                    0x1 => "LH",
                    0x2 => "LW",
                    0x3 => "LD",
                    0x4 => "LQ",
                    0x5 => "LHU",
                    0x6 => "LWU",
                    0x7 => "LDU",
                    _ => "L_UNKNOWN",
                }
            }

            OpCode::Store => {
                match decoded.funct3 {
                    0x0 => "SB",
                    0x1 => "SH",
                    0x2 => "SW",
                    0x3 => "SD",
                    0x4 => "SQ",
                    _ => "S_UNKNOWN",
                }
            }

            OpCode::Imm => {
                match decoded.funct3 {
                    0x0 => "ADDI",
                    0x1 => "SLLI",
                    0x2 => "SLTI",
                    0x3 => "SLTIU",
                    0x4 => "XORI",
                    0x5 => if (decoded.funct7 & 0x20) != 0 {"SRAI"} else {"SRLI"},
                    0x6 => "ORI",
                    0x7 => "ANDI",
                    _ => "I_UNKNOWN",
                }
            }

            OpCode::Reg => {
                match decoded.funct3 {
                    0x0 => {
                        if (decoded.funct7 & 0x20) != 0 {
                            "SUB"
                        } else {
                            "ADD"
                        }
                    },
                    0x1 => "SLL",
                    0x2 => "SLT",
                    0x3 => "SLTU",
                    0x4 => "XOR",
                    0x5 => if (decoded.funct7 & 0x20) != 0 {"SRA"} else {"SRL"},
                    0x6 => "OR",
                    0x7 => "AND",
                    _ => "R_UNKNOWN",
                }
            }

            OpCode::System => {
                if decoded.funct3 == 0x0 {
                    if decoded.imm == 0 {
                        return "ECALL";
                    }
                    if decoded.imm == 1 {
                        return "EBREAK";
                    }
                }
                "SYSTEM"
            }

            _ => "UNKNOWN"
        }
    }
}
