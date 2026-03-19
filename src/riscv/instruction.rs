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
    /// Atomic memory operations (原子内存操作 - A 扩展)
    Atomic = 0x2f,
    /// Floating-point load (浮点加载 - F/D/Q 扩展)
    FpLoad = 0x07,
    /// Floating-point store (浮点存储 - F/D/Q 扩展)
    FpStore = 0x27,
    /// Floating-point compute (浮点计算 - F/D/Q 扩展)
    FpCompute = 0x53,
    /// Vector operations (向量操作 - V 扩展)
    Vector = 0x57,
    /// Compressed instruction (16-bit, C 扩展) - special handling required
    Compressed = 0x00,
}

/// Floating-point format width for F/D/Q extensions.
///
/// ---
///
/// F/D/Q 扩展的浮点格式宽度。
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FpFormat {
    /// Single-precision (32-bit, F extension) (单精度，F 扩展)
    S = 0,
    /// Double-precision (64-bit, D extension) (双精度，D 扩展)
    D = 1,
    /// Quad-precision (128-bit, Q extension) (四精度，Q 扩展)
    Q = 3,
    /// Half-precision (16-bit, Zfh extension) (半精度，Zfh 扩展)
    H = 2,
}

impl FpFormat {
    /// Creates FpFormat from the fmt field in FP instructions.
    ///
    /// ---
    ///
    /// 从浮点指令的 fmt 字段创建 FpFormat。
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => FpFormat::S,
            1 => FpFormat::D,
            2 => FpFormat::H,
            3 => FpFormat::Q,
            _ => FpFormat::S, // Default to single precision
        }
    }

    /// Returns the size in bytes for this format.
    ///
    /// ---
    ///
    /// 返回此格式的字节大小。
    pub fn size_bytes(&self) -> u8 {
        match self {
            FpFormat::H => 2,
            FpFormat::S => 4,
            FpFormat::D => 8,
            FpFormat::Q => 16,
        }
    }

    /// Returns the suffix for instruction mnemonics.
    ///
    /// ---
    ///
    /// 返回指令助记符的后缀。
    pub fn suffix(&self) -> &'static str {
        match self {
            FpFormat::H => ".h",
            FpFormat::S => ".s",
            FpFormat::D => ".d",
            FpFormat::Q => ".q",
        }
    }
}

/// Floating-point rounding mode.
///
/// Uses the same encoding as FCSR.rm field.
///
/// ---
///
/// 浮点舍入模式。
///
/// 使用与 FCSR.rm 字段相同的编码。
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FpRoundingMode {
    /// Round to Nearest, ties to Even (向最近偶数舍入)
    Rne = 0,
    /// Round toward Zero (向零舍入)
    Rtz = 1,
    /// Round toward Down (向下舍入)
    Rdn = 2,
    /// Round toward Up (向上舍入)
    Rup = 3,
    /// Round to Nearest, ties to Max Magnitude (向最大幅值舍入)
    Rmm = 4,
    /// Dynamic (use FCSR.rm) (动态，使用 FCSR.rm)
    Dyn = 7,
}

impl FpRoundingMode {
    /// Creates FpRoundingMode from instruction rm field.
    ///
    /// ---
    ///
    /// 从指令 rm 字段创建 FpRoundingMode。
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => FpRoundingMode::Rne,
            1 => FpRoundingMode::Rtz,
            2 => FpRoundingMode::Rdn,
            3 => FpRoundingMode::Rup,
            4 => FpRoundingMode::Rmm,
            7 => FpRoundingMode::Dyn,
            _ => FpRoundingMode::Dyn,
        }
    }
}

/// Floating-point load funct3 codes.
///
/// ---
///
/// 浮点加载 funct3 代码。
#[repr(u8)]
pub(crate) enum FpLoadFunct3 {
    /// Load single-precision (加载单精度)
    Flw = 0x2,
    /// Load double-precision (加载双精度)
    Fld = 0x3,
    /// Load quad-precision (加载四精度)
    Flq = 0x4,
}

/// Floating-point store funct3 codes.
///
/// ---
///
/// 浮点存储 funct3 代码。
#[repr(u8)]
pub(crate) enum FpStoreFunct3 {
    /// Store single-precision (存储单精度)
    Fsw = 0x2,
    /// Store double-precision (存储双精度)
    Fsd = 0x3,
    /// Store quad-precision (存储四精度)
    Fsq = 0x4,
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

/// Atomic memory operation function 3 codes.
///
/// All AMO instructions use funct3 = 0x2.
///
/// ---
///
/// 原子内存操作 funct3 代码。
///
/// 所有 AMO 指令使用 funct3 = 0x2。
#[repr(u8)]
pub(crate) enum AtomicFunct3 {
    /// AMO operations (原子操作)
    Amo = 0x2,
}

/// Atomic memory operation function 5 codes (bits 31-27 of instruction).
///
/// Determines the specific AMO operation.
///
/// ---
///
/// 原子内存操作 funct5 代码（指令位 31-27）。
///
/// 决定具体的 AMO 操作。
#[repr(u8)]
pub(crate) enum AmoFunct5 {
    /// Load Reserved (加载保留)
    Lr = 0x02,
    /// Store Conditional (存储条件)
    Sc = 0x03,
    /// AMO Add (原子加法)
    AmoAdd = 0x00,
    /// AMO Swap (原子交换)
    AmoSwap = 0x01,
    /// AMO Logical AND (原子逻辑与)
    AmoAnd = 0x0c,
    /// AMO Logical OR (原子逻辑或)
    AmoOr = 0x0a,
    /// AMO Logical XOR (原子逻辑异或)
    AmoXor = 0x04,
    /// AMO Maximum signed (原子有符号最大值)
    AmoMax = 0x18,
    /// AMO Maximum unsigned (原子无符号最大值)
    AmoMaxu = 0x1c,
    /// AMO Minimum signed (原子有符号最小值)
    AmoMin = 0x10,
    /// AMO Minimum unsigned (原子无符号最小值)
    AmoMinu = 0x14,
}

/// Extracts funct5 from a 32-bit instruction (bits 31-27).
///
/// Used for AMO instructions to determine the operation.
///
/// ---
///
/// 从 32 位指令提取 funct5（位 31-27）。
///
/// 用于 AMO 指令确定操作。
pub fn extract_funct5(instruction: u32) -> u8 {
    ((instruction >> 27) & 0x1f) as u8
}

/// Checks if an AMO instruction has the acquire bit set (bit 26).
///
/// The acquire bit ensures that memory operations before the AMO
/// are visible to other harts.
///
/// ---
///
/// 检查 AMO 指令是否设置了 acquire 位（位 26）。
///
/// acquire 位确保 AMO 之前的内存操作对其他硬件线程可见。
pub fn extract_aq(instruction: u32) -> bool {
    ((instruction >> 26) & 0x1) != 0
}

/// Checks if an AMO instruction has the release bit set (bit 25).
///
/// The release bit ensures that memory operations after the AMO
/// are not visible to other harts before the AMO completes.
///
/// ---
///
/// 检查 AMO 指令是否设置了 release 位（位 25）。
///
/// release 位确保 AMO 之后的内存操作在 AMO 完成前对其他硬件线程不可见。
pub fn extract_rl(instruction: u32) -> bool {
    ((instruction >> 25) & 0x1) != 0
}

/// Floating-point compute operation funct7 values.
///
/// ---
///
/// 浮点计算操作 funct7 值。
#[repr(u8)]
pub(crate) enum FpComputeFunct7 {
    /// FADD.S/D/Q (浮点加法)
    Fadd = 0x00,
    /// FSUB.S/D/Q (浮点减法)
    Fsub = 0x04,
    /// FMUL.S/D/Q (浮点乘法)
    Fmul = 0x08,
    /// FDIV.S/D/Q (浮点除法)
    Fdiv = 0x0c,
    /// FSQRT.S/D/Q (浮点平方根)
    Fsqrt = 0x2c,
    /// FSGNJ.S/D/Q variants (浮点符号注入)
    Fsgnj = 0x10,
    /// FMIN/MAX.S/D/Q (浮点最小/最大值)
    Fminmax = 0x14,
    /// FCVT.S/D/Q <-> W/WD/L/LD (浮点-整数转换)
    Fcvt = 0x60,
    /// FMV/FCLASS (浮点移动/分类)
    FmvFclass = 0x70,
    /// FCVT.S <-> D (单精度-双精度转换)
    FcvtSd = 0x20,
    /// FCOMP (浮点比较) - encoded in rs2 field
    Fcmp = 0x50,
}

/// Extracts the fmt field from floating-point instructions.
///
/// For most FP compute instructions, fmt is in bits [26:25].
///
/// ---
///
/// 从浮点指令提取 fmt 字段。
///
/// 对于大多数浮点计算指令，fmt 位于位 [26:25]。
pub fn extract_fp_fmt(instruction: u32) -> u8 {
    ((instruction >> 25) & 0x3) as u8
}

/// Extracts the rm (rounding mode) field from floating-point instructions.
///
/// rm is in bits [14:12] for most FP instructions.
///
/// ---
///
/// 从浮点指令提取 rm（舍入模式）字段。
///
/// rm 对于大多数浮点指令位于位 [14:12]。
pub fn extract_fp_rm(instruction: u32) -> u8 {
    ((instruction >> 12) & 0x7) as u8
}

/// Determines the floating-point operation from funct7.
///
/// Returns a tuple of (funct7_high, fmt) where:
/// - funct7_high is bits [31:27]
/// - fmt is bits [26:25]
///
/// ---
///
/// 从 funct7 确定浮点操作。
///
/// 返回 (funct7_high, fmt) 元组，其中：
/// - funct7_high 是位 [31:27]
/// - fmt 是位 [26:25]
pub fn extract_fp_funct7_high(instruction: u32) -> u8 {
    ((instruction >> 27) & 0x1f) as u8
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
            OpCode::Jalr | OpCode::Load | OpCode::Imm | OpCode::System | OpCode::FpLoad => InstructionType::IType,
            OpCode::Store | OpCode::FpStore => InstructionType::SType,
            OpCode::Branch => InstructionType::BType,
            OpCode::Reg | OpCode::Atomic | OpCode::FpCompute | OpCode::Vector => InstructionType::RType,
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
            0x2f => OpCode::Atomic,
            0x07 => OpCode::FpLoad,    // Floating-point load
            0x27 => OpCode::FpStore,   // Floating-point store
            0x53 => OpCode::FpCompute, // Floating-point compute
            0x57 => OpCode::Vector,    // Vector operations (V extension)
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

    /// Checks if a 16-bit instruction is a compressed instruction.
    ///
    /// Returns true if the instruction is a valid C extension instruction.
    ///
    /// ---
    ///
    /// 检查 16 位指令是否为压缩指令。
    ///
    /// 如果指令是有效的 C 扩展指令则返回 true。
    pub fn is_compressed(instruction: u16) -> bool {
        let opcode = instruction & 0x3;
        // C extension opcodes are 0, 1, 2 (not 3 which is 32-bit)
        opcode != 0x3
    }

    /// Decodes a 16-bit compressed instruction.
    ///
    /// Returns a DecodedInstruction for the expanded 32-bit equivalent.
    ///
    /// ---
    ///
    /// 解码 16 位压缩指令。
    ///
    /// 返回扩展的 32 位等效指令的 DecodedInstruction。
    pub fn decode_compressed(instruction: u16) -> DecodedInstruction {
        let mut decoded = DecodedInstruction::new();
        
        let opcode = (instruction & 0x3) as u8;
        let funct3 = ((instruction >> 13) & 0x7) as u8;
        let rd = ((instruction >> 7) & 0x1f) as u8;
        let _rs1 = ((instruction >> 7) & 0x1f) as u8;
        let rs2 = ((instruction >> 2) & 0x1f) as u8;
        
        // Quadrant determines the instruction format
        match opcode {
            // Quadrant 0: C.ADDI4SPN, C.FLD, C.LW, C.FLW, C.LD, etc.
            0x00 => {
                decoded.funct3 = funct3;
                match funct3 {
                    0x0 => {
                        // C.ADDI4SPN (addi rd', x2, nzuimm)
                        let nzuimm = (((instruction >> 3) & 0x1f) | 
                                      (((instruction >> 5) & 0x1) << 5) |
                                      (((instruction >> 6) & 0x1) << 6) |
                                      (((instruction >> 11) & 0x1) << 7)) as i128;
                        decoded.opcode = OpCode::Imm;
                        decoded.rd = rd + 8;
                        decoded.rs1 = 2;
                        decoded.imm = nzuimm;
                    }
                    0x2 => {
                        // C.LW (lw rd', offset(rs1'))
                        decoded.opcode = OpCode::Load;
                        decoded.funct3 = 0x2;
                        decoded.rd = rd + 8;
                        decoded.rs1 = rs2 + 8;
                        let offset = ((((instruction >> 6) & 0x7) << 3) |
                                     (((instruction >> 10) & 0x3) << 1) |
                                     (((instruction >> 5) & 0x1) << 6)) as i128;
                        decoded.imm = offset;
                    }
                    0x3 => {
                        // C.LD (ld rd', offset(rs1')) - RV128
                        decoded.opcode = OpCode::Load;
                        decoded.funct3 = 0x3;
                        decoded.rd = rd + 8;
                        decoded.rs1 = rs2 + 8;
                        let offset = ((((instruction >> 6) & 0x7) << 3) |
                                     (((instruction >> 10) & 0x7) << 6)) as i128;
                        decoded.imm = offset;
                    }
                    0x6 => {
                        // C.SW (sw rs2', offset(rs1'))
                        decoded.opcode = OpCode::Store;
                        decoded.funct3 = 0x2;
                        decoded.rs1 = rd + 8;
                        decoded.rs2 = rs2 + 8;
                        let offset = ((((instruction >> 6) & 0x7) << 3) |
                                     (((instruction >> 10) & 0x3) << 1) |
                                     (((instruction >> 5) & 0x1) << 6)) as i128;
                        decoded.imm = offset;
                    }
                    0x7 => {
                        // C.SD (sd rs2', offset(rs1')) - RV128
                        decoded.opcode = OpCode::Store;
                        decoded.funct3 = 0x3;
                        decoded.rs1 = rd + 8;
                        decoded.rs2 = rs2 + 8;
                        let offset = ((((instruction >> 6) & 0x7) << 3) |
                                     (((instruction >> 10) & 0x7) << 6)) as i128;
                        decoded.imm = offset;
                    }
                    _ => {
                        decoded.opcode = OpCode::System;
                    }
                }
            }
            // Quadrant 1: C.ADDI, C.JAL, C.LI, C.ADDI16SP, etc.
            0x01 => {
                decoded.funct3 = funct3;
                match funct3 {
                    0x0 => {
                        // C.NOP / C.ADDI (addi rd, x0, imm / addi rd, rd, nzimm)
                        let imm = ((((instruction >> 12) & 0x1) << 5) |
                                  ((instruction >> 2) & 0x1f)) as i8 as i128;
                        decoded.opcode = OpCode::Imm;
                        decoded.rd = rd;
                        decoded.rs1 = rd;
                        decoded.imm = imm;
                    }
                    0x1 => {
                        // C.JAL (jal x1, offset)
                        let offset = (((instruction >> 2) & 0x1) |
                                     (((instruction >> 3) & 0x7) << 1) |
                                     (((instruction >> 6) & 0x1) << 4) |
                                     (((instruction >> 7) & 0x1) << 5) |
                                     (((instruction >> 8) & 0x3) << 6) |
                                     (((instruction >> 10) & 0x1) << 8) |
                                     (((instruction >> 11) & 0x1) << 9) |
                                     (((instruction >> 12) & 0x1) << 10)) as i16 as i128;
                        decoded.opcode = OpCode::Jal;
                        decoded.rd = 1;
                        decoded.imm = offset << 1;
                    }
                    0x2 => {
                        // C.LI (addi rd, x0, imm)
                        let imm = ((((instruction >> 12) & 0x1) << 5) |
                                  ((instruction >> 2) & 0x1f)) as i8 as i128;
                        decoded.opcode = OpCode::Imm;
                        decoded.rd = rd;
                        decoded.rs1 = 0;
                        decoded.imm = imm;
                    }
                    0x3 => {
                        // C.ADDI16SP / C.LUI
                        if rd == 2 {
                            // C.ADDI16SP (addi x2, x2, nzimm)
                            let nzimm = (((((instruction >> 12) & 0x1) << 9) |
                                         (((instruction >> 3) & 0x3) << 7) |
                                         (((instruction >> 5) & 0x1) << 6) |
                                         (((instruction >> 2) & 0x1) << 5) |
                                         (((instruction >> 6) & 0x1) << 4)) as i16) << 4;
                            decoded.opcode = OpCode::Imm;
                            decoded.rd = 2;
                            decoded.rs1 = 2;
                            decoded.imm = nzimm as i128;
                        } else {
                            // C.LUI (lui rd, nzimm)
                            let nzimm = ((((instruction >> 12) & 0x1) as u32) << 17) |
                                       ((((instruction >> 2) & 0x1f) as u32) << 12);
                            decoded.opcode = OpCode::Lui;
                            decoded.rd = rd;
                            decoded.imm = nzimm as i128;
                        }
                    }
                    0x4 => {
                        // C.SRLI, C.SRAI, C.ANDI, C.SUB, C.XOR, C.OR, C.AND
                        let funct2 = ((instruction >> 10) & 0x3) as u8;
                        match funct2 {
                            0x0 | 0x1 => {
                                // C.SRLI / C.SRAI
                                let shamt = ((((instruction >> 12) & 0x1) << 5) |
                                            ((instruction >> 2) & 0x1f)) as i128;
                                decoded.opcode = OpCode::Imm;
                                decoded.funct3 = 0x5;
                                decoded.rd = rs2 + 8;
                                decoded.rs1 = rs2 + 8;
                                decoded.imm = shamt;
                                decoded.funct7 = if funct2 == 0x1 { 0x20 } else { 0 };
                            }
                            0x2 => {
                                // C.ANDI
                                let imm = ((((instruction >> 12) & 0x1) << 5) |
                                          ((instruction >> 2) & 0x1f)) as i8 as i128;
                                decoded.opcode = OpCode::Imm;
                                decoded.funct3 = 0x7;
                                decoded.rd = rs2 + 8;
                                decoded.rs1 = rs2 + 8;
                                decoded.imm = imm;
                            }
                            0x3 => {
                                // C.SUB, C.XOR, C.OR, C.AND
                                decoded.opcode = OpCode::Reg;
                                decoded.rs1 = rs2 + 8;
                                decoded.rs2 = rd;
                                decoded.rd = rs2 + 8;
                                let funct2_sub = ((instruction >> 5) & 0x3) as u8;
                                decoded.funct3 = match funct2_sub {
                                    0x0 => 0x0, // C.SUB
                                    0x1 => 0x4, // C.XOR
                                    0x2 => 0x6, // C.OR
                                    0x3 => 0x7, // C.AND
                                    _ => 0x0,
                                };
                                decoded.funct7 = if funct2_sub == 0 { 0x20 } else { 0 };
                            }
                            _ => {}
                        }
                    }
                    0x5 => {
                        // C.J (jal x0, offset)
                        let offset = (((instruction >> 2) & 0x1) |
                                     (((instruction >> 3) & 0x7) << 1) |
                                     (((instruction >> 6) & 0x1) << 4) |
                                     (((instruction >> 7) & 0x1) << 5) |
                                     (((instruction >> 8) & 0x3) << 6) |
                                     (((instruction >> 10) & 0x1) << 8) |
                                     (((instruction >> 11) & 0x1) << 9) |
                                     (((instruction >> 12) & 0x1) << 10)) as i16 as i128;
                        decoded.opcode = OpCode::Jal;
                        decoded.rd = 0;
                        decoded.imm = offset << 1;
                    }
                    0x6 => {
                        // C.BEQZ (beq rs1', x0, offset)
                        let offset = (((instruction >> 2) & 0x1) |
                                     (((instruction >> 3) & 0x3) << 1) |
                                     (((instruction >> 5) & 0x3) << 3) |
                                     (((instruction >> 10) & 0x3) << 5) |
                                     (((instruction >> 12) & 0x1) << 7)) as i16 as i128;
                        decoded.opcode = OpCode::Branch;
                        decoded.funct3 = 0x0;
                        decoded.rs1 = rs2 + 8;
                        decoded.rs2 = 0;
                        decoded.imm = offset << 1;
                    }
                    0x7 => {
                        // C.BNEZ (bne rs1', x0, offset)
                        let offset = (((instruction >> 2) & 0x1) |
                                     (((instruction >> 3) & 0x3) << 1) |
                                     (((instruction >> 5) & 0x3) << 3) |
                                     (((instruction >> 10) & 0x3) << 5) |
                                     (((instruction >> 12) & 0x1) << 7)) as i16 as i128;
                        decoded.opcode = OpCode::Branch;
                        decoded.funct3 = 0x1;
                        decoded.rs1 = rs2 + 8;
                        decoded.rs2 = 0;
                        decoded.imm = offset << 1;
                    }
                    _ => {}
                }
            }
            // Quadrant 2: C.SLLI, C.FLDSP, C.LWSP, C.FLWSP, C.LDSP, etc.
            0x02 => {
                decoded.funct3 = funct3;
                match funct3 {
                    0x0 => {
                        // C.SLLI (slli rd, rd, shamt)
                        let shamt = ((((instruction >> 12) & 0x1) << 5) |
                                    ((instruction >> 2) & 0x1f)) as i128;
                        decoded.opcode = OpCode::Imm;
                        decoded.funct3 = 0x1;
                        decoded.rd = rd;
                        decoded.rs1 = rd;
                        decoded.imm = shamt;
                    }
                    0x2 => {
                        // C.LWSP (lw rd, offset(x2))
                        let offset = ((((instruction >> 2) & 0x7) << 2) |
                                     (((instruction >> 12) & 0x1) << 5) |
                                     (((instruction >> 5) & 0x3) << 6)) as i128;
                        decoded.opcode = OpCode::Load;
                        decoded.funct3 = 0x2;
                        decoded.rd = rd;
                        decoded.rs1 = 2;
                        decoded.imm = offset;
                    }
                    0x3 => {
                        // C.LDSP (ld rd, offset(x2)) - RV128
                        let offset = ((((instruction >> 2) & 0x7) << 3) |
                                     (((instruction >> 12) & 0x1) << 5) |
                                     (((instruction >> 5) & 0x7) << 6)) as i128;
                        decoded.opcode = OpCode::Load;
                        decoded.funct3 = 0x3;
                        decoded.rd = rd;
                        decoded.rs1 = 2;
                        decoded.imm = offset;
                    }
                    0x4 => {
                        // C.JR, C.MV, C.EBREAK, C.JALR, C.ADD
                        if ((instruction >> 12) & 0x1) == 0 {
                            if rs2 == 0 {
                                // C.JR (jalr x0, rs1, 0)
                                decoded.opcode = OpCode::Jalr;
                                decoded.rd = 0;
                                decoded.rs1 = rd;
                                decoded.imm = 0;
                            } else {
                                // C.MV (add rd, x0, rs2)
                                decoded.opcode = OpCode::Reg;
                                decoded.funct3 = 0x0;
                                decoded.rd = rd;
                                decoded.rs1 = 0;
                                decoded.rs2 = rs2;
                            }
                        } else {
                            if rd == 0 && rs2 == 0 {
                                // C.EBREAK
                                decoded.opcode = OpCode::System;
                                decoded.imm = 1;
                            } else if rs2 == 0 {
                                // C.JALR (jalr x1, rs1, 0)
                                decoded.opcode = OpCode::Jalr;
                                decoded.rd = 1;
                                decoded.rs1 = rd;
                                decoded.imm = 0;
                            } else {
                                // C.ADD (add rd, rd, rs2)
                                decoded.opcode = OpCode::Reg;
                                decoded.funct3 = 0x0;
                                decoded.rd = rd;
                                decoded.rs1 = rd;
                                decoded.rs2 = rs2;
                            }
                        }
                    }
                    0x6 => {
                        // C.SWSP (sw rs2, offset(x2))
                        let offset = ((((instruction >> 2) & 0x7) << 2) |
                                     (((instruction >> 9) & 0xf) << 5)) as i128;
                        decoded.opcode = OpCode::Store;
                        decoded.funct3 = 0x2;
                        decoded.rs1 = 2;
                        decoded.rs2 = rs2;
                        decoded.imm = offset;
                    }
                    0x7 => {
                        // C.SDSP (sd rs2, offset(x2)) - RV128
                        let offset = ((((instruction >> 2) & 0x7) << 3) |
                                     (((instruction >> 10) & 0x7) << 6) |
                                     (((instruction >> 9) & 0x1) << 9)) as i128;
                        decoded.opcode = OpCode::Store;
                        decoded.funct3 = 0x3;
                        decoded.rs1 = 2;
                        decoded.rs2 = rs2;
                        decoded.imm = offset;
                    }
                    _ => {}
                }
            }
            _ => {
                decoded.opcode = OpCode::System;
            }
        }
        
        decoded.typ = Self::get_instruction_type(&decoded.opcode, decoded.funct3);
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
                // Check for Zba/Zbb immediate instructions
                if decoded.funct7 == 0x0C {
                    // Zbb RORI
                    match decoded.funct3 {
                        0x5 => "RORI",
                        _ => "ZBB_IMM_UNKNOWN",
                    }
                } else if decoded.funct7 == 0x08 {
                    // Zba SLLI.UW
                    match decoded.funct3 {
                        0x1 => "SLLI.UW",
                        _ => "ZBA_IMM_UNKNOWN",
                    }
                } else if decoded.funct3 == 0x5 && decoded.funct7 == 0x20 {
                    // Zbb BEXTI (bit extract immediate)
                    "BEXTI"
                } else if decoded.funct3 == 0x1 && decoded.funct7 == 0x30 {
                    // Zbb BCLRI (bit clear immediate)
                    "BCLRI"
                } else if decoded.funct3 == 0x1 && decoded.funct7 == 0x28 {
                    // Zbb BSETI (bit set immediate)
                    "BSETI"
                } else if decoded.funct3 == 0x5 && decoded.funct7 == 0x28 {
                    // Zbb BINVI (bit invert immediate)
                    "BINVI"
                } else {
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
            }

            OpCode::Reg => {
                // Check for M extension instructions (funct7 == 0x01)
                if decoded.funct7 == 0x01 {
                    match decoded.funct3 {
                        0x0 => "MUL",
                        0x1 => "MULH",
                        0x2 => "MULHSU",
                        0x3 => "MULHU",
                        0x4 => "DIV",
                        0x5 => "DIVU",
                        0x6 => "REM",
                        0x7 => "REMU",
                        _ => "M_UNKNOWN",
                    }
                } else if (decoded.funct7 & 0x80) != 0 {
                    // P extension: SIMD instructions have bit 7 set in funct7
                    Self::get_simd_instruction_name(decoded.funct7, decoded.funct3)
                } else if decoded.funct7 == 0x04 {
                    // Zba/Zbb instructions (funct7 = 0x04)
                    match decoded.funct3 {
                        0x0 => "SH1ADD",  // Zba
                        0x1 => "ROL",     // Zbb
                        0x2 => "SH2ADD",  // Zba
                        0x3 => "ROR",     // Zbb (also used by Zbkb)
                        0x4 => "SH3ADD",  // Zba
                        0x5 => {
                            // For immediate variants, check rs2/immediate
                            // This is for register versions - could be FSRI, but we use ROR
                            "ROR"
                        },
                        0x6 => "ROR",     // Zbb alternative encoding
                        0x7 => "ANDN",    // Zbb
                        _ => "ZBA_ZBB_UNKNOWN",
                    }
                } else if decoded.funct7 == 0x05 {
                    // Zbb/Zbc instructions (funct7 = 0x05)
                    match decoded.funct3 {
                        0x0 => "CLZ",     // Zbb (count leading zeros)
                        0x1 => "CTZ",     // Zbb (count trailing zeros)
                        0x2 => "CPOP",    // Zbb (population count)
                        0x3 => "CLMUL",   // Zbc (carry-less multiply)
                        0x4 => "CLMULR",  // Zbc
                        0x5 => "CLMULH",  // Zbc
                        0x6 => "MIN",     // Zbb alternative
                        0x7 => "MAX",     // Zbb alternative
                        _ => "ZBB_ZBC_UNKNOWN",
                    }
                } else if decoded.funct7 == 0x06 {
                    // Zbb additional instructions (funct7 = 0x06)
                    match decoded.funct3 {
                        0x0 => "MINU",    // Zbb
                        0x1 => "MAXU",    // Zbb
                        0x2 => "MIN",     // Zbb
                        0x3 => "MAX",     // Zbb
                        0x5 => "ORN",     // Zbb
                        0x6 => "XORN",    // Zbb
                        _ => "ZBB_UNKNOWN",
                    }
                } else if decoded.funct7 == 0x20 {
                    // Zba.UW instructions (funct7 = 0x20)
                    match decoded.funct3 {
                        0x0 => "SH1ADD.UW",  // Zba
                        0x2 => "SH2ADD.UW",  // Zba
                        0x4 => "SH3ADD.UW",  // Zba
                        _ => "ZBA_UW_UNKNOWN",
                    }
                } else if decoded.funct7 == 0x30 {
                    // Zba ADD.UW instruction
                    match decoded.funct3 {
                        0x0 => "ADD.UW",  // Zba
                        _ => "ZBA_ADD_UW_UNKNOWN",
                    }
                } else if decoded.funct7 == 0x0C {
                    // Zbb RORI and similar (funct7 = 0x0C for immediate forms)
                    match decoded.funct3 {
                        0x5 => "RORI",    // Zbb
                        _ => "ZBB_IMM_UNKNOWN",
                    }
                } else if decoded.funct7 == 0x08 {
                    // Zba SLLI.UW instruction
                    match decoded.funct3 {
                        0x1 => "SLLI.UW", // Zba
                        _ => "ZBA_SLLI_UW_UNKNOWN",
                    }
                } else {
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
            }

            OpCode::MiscMem => {
                // Zifencei extension: FENCE.I
                // MISC-MEM instructions: FENCE, FENCE.I
                match decoded.funct3 {
                    0x0 => "FENCE",
                    0x1 => "FENCE.I",
                    _ => "MISCMEM_UNKNOWN",
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
                // Zicsr extension: CSR instructions
                match decoded.funct3 {
                    0x1 => "CSRRW",
                    0x2 => "CSRRS",
                    0x3 => "CSRRC",
                    0x5 => "CSRRWI",
                    0x6 => "CSRRSI",
                    0x7 => "CSRRCI",
                    _ => "SYSTEM",
                }
            }

            OpCode::Atomic => {
                // AMO instructions use funct5 (bits 31-27) stored in upper bits of funct7
                // funct7 format for AMO: funct5(5) | aq(1) | rl(1)
                let funct5 = (decoded.funct7 >> 2) & 0x1f;
                match funct5 {
                    0x02 => "LR.D",
                    0x03 => "SC.D",
                    0x00 => "AMOADD.D",
                    0x01 => "AMOSWAP.D",
                    0x0c => "AMOAND.D",
                    0x0a => "AMOOR.D",
                    0x04 => "AMOXOR.D",
                    0x18 => "AMOMAX.D",
                    0x1c => "AMOMAXU.D",
                    0x10 => "AMOMIN.D",
                    0x14 => "AMOMINU.D",
                    _ => "AMO_UNKNOWN",
                }
            }

            OpCode::FpLoad => {
                // Floating-point load: FLW, FLD, FLQ
                match decoded.funct3 {
                    0x2 => "FLW",
                    0x3 => "FLD",
                    0x4 => "FLQ",
                    _ => "FP_LOAD_UNKNOWN",
                }
            }

            OpCode::FpStore => {
                // Floating-point store: FSW, FSD, FSQ
                match decoded.funct3 {
                    0x2 => "FSW",
                    0x3 => "FSD",
                    0x4 => "FSQ",
                    _ => "FP_STORE_UNKNOWN",
                }
            }

            OpCode::FpCompute => {
                // Floating-point compute instructions
                // funct7 format: funct5(5) | fmt(2) where fmt indicates precision
                let funct5 = (decoded.funct7 >> 2) & 0x1f;
                let fmt = decoded.funct7 & 0x3;
                
                // Determine suffix based on fmt
                let suffix = match fmt {
                    0 => ".S",  // Single precision
                    1 => ".D",  // Double precision
                    3 => ".Q",  // Quad precision
                    _ => ".?",
                };
                
                match funct5 {
                    // FADD
                    0x00 => return Self::fp_name_with_suffix("FADD", suffix),
                    // FSUB
                    0x04 => return Self::fp_name_with_suffix("FSUB", suffix),
                    // FMUL
                    0x08 => return Self::fp_name_with_suffix("FMUL", suffix),
                    // FDIV
                    0x0c => return Self::fp_name_with_suffix("FDIV", suffix),
                    // FSQRT
                    0x0b => return Self::fp_name_with_suffix("FSQRT", suffix),
                    // FSGNJ, FSGNJN, FSGNJX (funct5 = 0x10)
                    0x10 => match decoded.funct3 {
                        0x0 => return Self::fp_name_with_suffix("FSGNJ", suffix),
                        0x1 => return Self::fp_name_with_suffix("FSGNJN", suffix),
                        0x2 => return Self::fp_name_with_suffix("FSGNJX", suffix),
                        _ => "FSGNJ_UNKNOWN",
                    },
                    // FMIN, FMAX
                    0x05 => match decoded.funct3 {
                        0x0 => return Self::fp_name_with_suffix("FMIN", suffix),
                        0x1 => return Self::fp_name_with_suffix("FMAX", suffix),
                        _ => "FMINMAX_UNKNOWN",
                    },
                    // FCVT.W.S/D/Q, FCVT.WU.S/D/Q, FCVT.L.S/D/Q, FCVT.LU.S/D/Q
                    0x18 => {
                        let src_fmt = decoded.rs2 & 0x7;
                        let dst_fmt = (decoded.rs2 >> 3) & 0x7;
                        return Self::get_fcvt_int_name(src_fmt, dst_fmt, decoded.rs2);
                    }
                    // FCVT.S/D/Q <-> S/D/Q (funct5 = 0x08 but with rs2 indicating conversion)
                    // Note: This conflicts with FMUL in a simple match, need to check rs2
                    // FMUL uses rs2 as a register, FCVT uses rs2 as format encoding
                    // For now, we handle FMUL above; this branch is unreachable but kept for reference
                    // FMV.X.S/D/Q, FCLASS.S/D/Q
                    0x1c => match decoded.funct3 {
                        0x0 => {
                            if decoded.rs2 == 0 {
                                return Self::fp_name_with_suffix("FMV.X", suffix);
                            } else if decoded.rs2 == 1 {
                                return Self::fp_name_with_suffix("FCLASS", suffix);
                            }
                            "FMV_FCLASS_UNKNOWN"
                        }
                        0x1 => return Self::fp_name_with_suffix("FMV.X", suffix),
                        _ => "FMV_X_UNKNOWN",
                    },
                    // FMV.S/D/Q.X
                    0x1e => match decoded.funct3 {
                        0x0 => return Self::fp_name_with_suffix("FMV", suffix),
                        _ => "FMV_X_UNKNOWN",
                    },
                    // FEQ, FLT, FLE
                    0x14 => match decoded.funct3 {
                        0x0 => return Self::fp_name_with_suffix("FLE", suffix),
                        0x1 => return Self::fp_name_with_suffix("FLT", suffix),
                        0x2 => return Self::fp_name_with_suffix("FEQ", suffix),
                        _ => "FCMP_UNKNOWN",
                    },
                    _ => "FP_COMPUTE_UNKNOWN",
                }
            }

            OpCode::Vector => {
                // V extension vector instructions
                // Uses funct3 and funct7 to determine operation
                Self::get_vector_instruction_name(decoded)
            }

            _ => "UNKNOWN"
        }
    }

    /// Gets the SIMD instruction name for P extension.
    ///
    /// P extension SIMD instructions use funct7 with bit 7 set.
    /// The instruction format varies based on the specific operation.
    ///
    /// ---
    ///
    /// 获取 P 扩展的 SIMD 指令名称。
    ///
    /// P 扩展 SIMD 指令使用设置了位 7 的 funct7。
    /// 指令格式根据具体操作而变化。
    fn get_simd_instruction_name(funct7: u8, funct3: u8) -> &'static str {
        // P extension SIMD instructions
        // funct7[6:0] determines the operation, funct7[7] = 1 indicates SIMD
        let op = funct7 & 0x7f;
        
        match op {
            // SIMD Addition (并行加法)
            0x00 => match funct3 {
                0x0 => "ADD8",    // 8-bit SIMD add
                0x1 => "ADD16",   // 16-bit SIMD add
                0x2 => "ADD32",   // 32-bit SIMD add
                0x3 => "ADD64",   // 64-bit SIMD add
                _ => "SIMD_ADD_UNKNOWN",
            },
            // SIMD Subtraction (并行减法)
            0x02 => match funct3 {
                0x0 => "SUB8",
                0x1 => "SUB16",
                0x2 => "SUB32",
                0x3 => "SUB64",
                _ => "SIMD_SUB_UNKNOWN",
            },
            // SIMD Unsigned Addition with saturation (无符号饱和加法)
            0x04 => match funct3 {
                0x0 => "ADDU8",
                0x1 => "ADDU16",
                0x2 => "ADDU32",
                0x3 => "ADDU64",
                _ => "SIMD_ADDU_UNKNOWN",
            },
            // SIMD Signed Addition with saturation (有符号饱和加法)
            0x05 => match funct3 {
                0x0 => "ADDS8",
                0x1 => "ADDS16",
                0x2 => "ADDS32",
                0x3 => "ADDS64",
                _ => "SIMD_ADDS_UNKNOWN",
            },
            // SIMD Unsigned Subtraction with saturation (无符号饱和减法)
            0x06 => match funct3 {
                0x0 => "SUBU8",
                0x1 => "SUBU16",
                0x2 => "SUBU32",
                0x3 => "SUBU64",
                _ => "SIMD_SUBU_UNKNOWN",
            },
            // SIMD Signed Subtraction with saturation (有符号饱和减法)
            0x07 => match funct3 {
                0x0 => "SUBS8",
                0x1 => "SUBS16",
                0x2 => "SUBS32",
                0x3 => "SUBS64",
                _ => "SIMD_SUBS_UNKNOWN",
            },
            // SIMD Average (并行平均)
            0x08 => match funct3 {
                0x0 => "AVE8",
                0x1 => "AVE16",
                0x2 => "AVE32",
                0x3 => "AVE64",
                _ => "SIMD_AVE_UNKNOWN",
            },
            // SIMD Shift Left (并行左移)
            0x0a => match funct3 {
                0x0 => "SLL8",
                0x1 => "SLL16",
                0x2 => "SLL32",
                0x3 => "SLL64",
                _ => "SIMD_SLL_UNKNOWN",
            },
            // SIMD Shift Right Logical (并行逻辑右移)
            0x0c => match funct3 {
                0x0 => "SRL8",
                0x1 => "SRL16",
                0x2 => "SRL32",
                0x3 => "SRL64",
                _ => "SIMD_SRL_UNKNOWN",
            },
            // SIMD Shift Right Arithmetic (并行算术右移)
            0x0d => match funct3 {
                0x0 => "SRA8",
                0x1 => "SRA16",
                0x2 => "SRA32",
                0x3 => "SRA64",
                _ => "SIMD_SRA_UNKNOWN",
            },
            // SIMD Compare Equal (并行比较相等)
            0x10 => match funct3 {
                0x0 => "CMPEQ8",
                0x1 => "CMPEQ16",
                0x2 => "CMPEQ32",
                0x3 => "CMPEQ64",
                _ => "SIMD_CMPEQ_UNKNOWN",
            },
            // SIMD Compare Less Than (并行小于比较)
            0x12 => match funct3 {
                0x0 => "CMPLT8",
                0x1 => "CMPLT16",
                0x2 => "CMPLT32",
                0x3 => "CMPLT64",
                _ => "SIMD_CMPLT_UNKNOWN",
            },
            // SIMD Compare Less Than Unsigned (并行无符号小于比较)
            0x13 => match funct3 {
                0x0 => "CMPLTU8",
                0x1 => "CMPLTU16",
                0x2 => "CMPLTU32",
                0x3 => "CMPLTU64",
                _ => "SIMD_CMPLTU_UNKNOWN",
            },
            // SIMD Minimum (并行最小值)
            0x14 => match funct3 {
                0x0 => "MIN8",
                0x1 => "MIN16",
                0x2 => "MIN32",
                0x3 => "MIN64",
                _ => "SIMD_MIN_UNKNOWN",
            },
            // SIMD Minimum Unsigned (并行无符号最小值)
            0x15 => match funct3 {
                0x0 => "MINU8",
                0x1 => "MINU16",
                0x2 => "MINU32",
                0x3 => "MINU64",
                _ => "SIMD_MINU_UNKNOWN",
            },
            // SIMD Maximum (并行最大值)
            0x16 => match funct3 {
                0x0 => "MAX8",
                0x1 => "MAX16",
                0x2 => "MAX32",
                0x3 => "MAX64",
                _ => "SIMD_MAX_UNKNOWN",
            },
            // SIMD Maximum Unsigned (并行无符号最大值)
            0x17 => match funct3 {
                0x0 => "MAXU8",
                0x1 => "MAXU16",
                0x2 => "MAXU32",
                0x3 => "MAXU64",
                _ => "SIMD_MAXU_UNKNOWN",
            },
            // SIMD AND (并行与)
            0x18 => match funct3 {
                0x0 => "AND8",
                0x1 => "AND16",
                0x2 => "AND32",
                0x3 => "AND64",
                _ => "SIMD_AND_UNKNOWN",
            },
            // SIMD OR (并行或)
            0x1a => match funct3 {
                0x0 => "OR8",
                0x1 => "OR16",
                0x2 => "OR32",
                0x3 => "OR64",
                _ => "SIMD_OR_UNKNOWN",
            },
            // SIMD XOR (并行异或)
            0x1c => match funct3 {
                0x0 => "XOR8",
                0x1 => "XOR16",
                0x2 => "XOR32",
                0x3 => "XOR64",
                _ => "SIMD_XOR_UNKNOWN",
            },
            // SIMD Pack (并行打包)
            0x20 => match funct3 {
                0x0 => "PACK8",
                0x1 => "PACK16",
                0x2 => "PACK32",
                0x3 => "PACK64",
                _ => "SIMD_PACK_UNKNOWN",
            },
            // SIMD Pack Upper (并行打包高位)
            0x21 => match funct3 {
                0x0 => "PACKU8",
                0x1 => "PACKU16",
                0x2 => "PACKU32",
                0x3 => "PACKU64",
                _ => "SIMD_PACKU_UNKNOWN",
            },
            // SIMD Multiply (并行乘法)
            0x24 => match funct3 {
                0x0 => "MUL8",
                0x1 => "MUL16",
                0x2 => "MUL32",
                0x3 => "MUL64",
                _ => "SIMD_MUL_UNKNOWN",
            },
            // SIMD Multiply and Add (并行乘加)
            0x25 => match funct3 {
                0x0 => "MULA8",
                0x1 => "MULA16",
                0x2 => "MULA32",
                0x3 => "MULA64",
                _ => "SIMD_MULA_UNKNOWN",
            },
            // SIMD Multiply and Subtract (并行乘减)
            0x26 => match funct3 {
                0x0 => "MULS8",
                0x1 => "MULS16",
                0x2 => "MULS32",
                0x3 => "MULS64",
                _ => "SIMD_MULS_UNKNOWN",
            },
            // SIMD Dot Product (并行点积)
            0x28 => match funct3 {
                0x0 => "DOTP8",
                0x1 => "DOTP16",
                0x2 => "DOTP32",
                0x3 => "DOTP64",
                _ => "SIMD_DOTP_UNKNOWN",
            },
            // SIMD Shuffle (并行重排)
            0x30 => match funct3 {
                0x0 => "SHFL8",
                0x1 => "SHFL16",
                0x2 => "SHFL32",
                0x3 => "SHFL64",
                _ => "SIMD_SHFL_UNKNOWN",
            },
            // SIMD Unpack Lower (并行解包低位)
            0x32 => match funct3 {
                0x0 => "UNPK8",
                0x1 => "UNPK16",
                0x2 => "UNPK32",
                _ => "SIMD_UNPK_UNKNOWN",
            },
            // SIMD Unpack Upper (并行解包高位)
            0x33 => match funct3 {
                0x0 => "UNPKU8",
                0x1 => "UNPKU16",
                0x2 => "UNPKU32",
                _ => "SIMD_UNPKU_UNKNOWN",
            },
            _ => "SIMD_UNKNOWN",
        }
    }

    /// Gets the name for vector instructions.
    ///
    /// ---
    ///
    /// 获取向量指令名称。
    fn get_vector_instruction_name(decoded: &DecodedInstruction) -> &'static str {
        let funct3 = decoded.funct3;
        // funct6 is in bits [31:26], stored in upper bits of funct7
        let funct6 = (decoded.funct7 >> 1) & 0x3f;
        
        match funct3 {
            // OPIVV: Vector-Vector integer operations
            0x0 => match funct6 {
                0x00 => "VADD_VV",
                0x02 => "VSUB_VV",
                0x04 => "VMIN_VV",
                0x05 => "VMINU_VV",
                0x06 => "VMAX_VV",
                0x07 => "VMAXU_VV",
                0x08 => "VAND_VV",
                0x09 => "VOR_VV",
                0x0a => "VXOR_VV",
                0x0b => "VRGATHER_VV",
                0x0c => "VSLL_VV",
                0x0d => "VSRL_VV",
                0x0e => "VSRA_VV",
                0x10 => "VMSEQ_VV",
                0x11 => "VMSNE_VV",
                0x12 => "VMSLTU_VV",
                0x13 => "VMSLT_VV",
                0x14 => "VMSLE_VV",
                0x15 => "VMSLEU_VV",
                0x18 => "VMV_V_V",
                0x1a => "VMV_X_S",
                0x1b => "VMV_S_X",
                0x1c => "VMV1R_V",
                0x1d => "VMV2R_V",
                0x20 => "VADC_VVM",
                0x21 => "VMADC_VVM",
                0x22 => "VSBC_VVM",
                0x23 => "VMSBC_VVM",
                0x24 => "VMERGE_VVM",
                0x25 => "VMSEQ_VV",
                _ => "V_OPIVV_UNKNOWN",
            },
            // OPFVV: Vector-Vector floating-point operations
            0x1 => match funct6 {
                0x00 => "VFADD_VV",
                0x02 => "VFSUB_VV",
                0x04 => "VFMIN_VV",
                0x05 => "VFMAX_VV",
                0x08 => "VFSGNJ_VV",
                0x09 => "VFSGNJN_VV",
                0x0a => "VFSGNJX_VV",
                0x10 => "VMFEQ_VV",
                0x11 => "VMFNE_VV",
                0x12 => "VMFLT_VV",
                0x13 => "VMFLE_VV",
                0x14 => "VMFGT_VV",
                0x15 => "VMFGE_VV",
                0x18 => "VFMV_V_F",
                0x1a => "VFMV_F_S",
                0x1b => "VFMV_S_F",
                _ => "V_OPFVV_UNKNOWN",
            },
            // OPMVV: Vector-Vector multiply/divide operations
            0x2 => match funct6 {
                0x0c => "VDIVU_VV",
                0x0d => "VDIV_VV",
                0x0e => "VREMU_VV",
                0x0f => "VREM_VV",
                0x10 => "VMULHU_VV",
                0x11 => "VMUL_VV",
                0x12 => "VMULHSU_VV",
                0x13 => "VMULH_VV",
                0x14 => "VMULHU_VV",
                0x18 => "VMACC_VV",
                0x19 => "VNMSAC_VV",
                0x1a => "VMADD_VV",
                0x1b => "VNMSUB_VV",
                0x24 => "VWSUB_VV",
                0x25 => "VWSUBU_VV",
                0x26 => "VWMUL_VV",
                0x28 => "VWMACCU_VV",
                0x29 => "VWMACC_VV",
                0x2b => "VWMACCSU_VV",
                _ => "V_OPMVV_UNKNOWN",
            },
            // OPVI: Vector-Immediate integer operations
            0x3 => match funct6 {
                0x00 => "VADD_VX",
                0x02 => "VSUB_VX",
                0x04 => "VMIN_VX",
                0x05 => "VMINU_VX",
                0x06 => "VMAX_VX",
                0x07 => "VMAXU_VX",
                0x08 => "VAND_VX",
                0x09 => "VOR_VX",
                0x0a => "VXOR_VX",
                0x0c => "VSLL_VX",
                0x0d => "VSRL_VX",
                0x0e => "VSRA_VX",
                0x10 => "VMSEQ_VX",
                0x11 => "VMSNE_VX",
                0x14 => "VMSLE_VX",
                0x15 => "VMSLEU_VX",
                0x16 => "VMSGT_VX",
                0x17 => "VMSGTU_VX",
                _ => "V_OPVI_UNKNOWN",
            },
            // OPIVI: Vector-5-bit immediate integer operations
            0x4 => match funct6 {
                0x00 => "VADD_VI",
                0x08 => "VAND_VI",
                0x09 => "VOR_VI",
                0x0a => "VXOR_VI",
                0x0c => "VSLL_VI",
                0x0d => "VSRL_VI",
                0x0e => "VSRA_VI",
                0x0f => "VNSRL_VI",
                0x10 => "VMSEQ_VI",
                0x11 => "VMSNE_VI",
                0x14 => "VMSLE_VI",
                0x16 => "VMSGT_VI",
                0x17 => "VMSGTU_VI",
                0x20 => "VADC_VIM",
                0x21 => "VMADC_VIM",
                0x24 => "VMERGE_VIM",
                _ => "V_OPIVI_UNKNOWN",
            },
            // OPFVF: Vector-Scalar floating-point operations
            0x5 => match funct6 {
                0x00 => "VFADD_VF",
                0x02 => "VFSUB_VF",
                0x04 => "VFMIN_VF",
                0x05 => "VFMAX_VF",
                0x08 => "VFSGNJ_VF",
                0x09 => "VFSGNJN_VF",
                0x0a => "VFSGNJX_VF",
                0x10 => "VMFEQ_VF",
                0x12 => "VMFLT_VF",
                0x13 => "VMFLE_VF",
                0x14 => "VMFGT_VF",
                0x15 => "VMFGE_VF",
                0x18 => "VFMV_V_F",
                _ => "V_OPFVF_UNKNOWN",
            },
            // OPCFG: Vector configuration
            0x7 => match funct6 {
                0x30 => "VSETVLI",
                0x31 => "VSETIVLI",
                0x3f => "VSETVL",
                _ => "V_OPCFG_UNKNOWN",
            },
            // VL: Vector load
            0x6 => match funct6 {
                0x00 => "VLE8_V",
                0x01 => "VLE16_V",
                0x02 => "VLE32_V",
                0x03 => "VLE64_V",
                0x04 => "VLE128_V",
                0x05 => "VLE256_V",
                0x06 => "VLE512_V",
                0x07 => "VLE1024_V",
                0x20 => "VLSE8_V",
                0x21 => "VLSE16_V",
                0x22 => "VLSE32_V",
                0x23 => "VLSE64_V",
                0x40 => "VLXEI8_V",
                0x41 => "VLXEI16_V",
                0x42 => "VLXEI32_V",
                0x43 => "VLXEI64_V",
                _ => "V_VL_UNKNOWN",
            },
            _ => "V_UNKNOWN",
        }
    }

    /// Helper function to create FP instruction name with suffix.
    ///
    /// ---
    ///
    /// 辅助函数，创建带后缀的浮点指令名称。
    fn fp_name_with_suffix(base: &'static str, suffix: &str) -> &'static str {
        // Return a static string based on the combination
        match (base, suffix) {
            ("FADD", ".S") => "FADD.S",
            ("FADD", ".D") => "FADD.D",
            ("FADD", ".Q") => "FADD.Q",
            ("FSUB", ".S") => "FSUB.S",
            ("FSUB", ".D") => "FSUB.D",
            ("FSUB", ".Q") => "FSUB.Q",
            ("FMUL", ".S") => "FMUL.S",
            ("FMUL", ".D") => "FMUL.D",
            ("FMUL", ".Q") => "FMUL.Q",
            ("FDIV", ".S") => "FDIV.S",
            ("FDIV", ".D") => "FDIV.D",
            ("FDIV", ".Q") => "FDIV.Q",
            ("FSQRT", ".S") => "FSQRT.S",
            ("FSQRT", ".D") => "FSQRT.D",
            ("FSQRT", ".Q") => "FSQRT.Q",
            ("FSGNJ", ".S") => "FSGNJ.S",
            ("FSGNJ", ".D") => "FSGNJ.D",
            ("FSGNJ", ".Q") => "FSGNJ.Q",
            ("FSGNJN", ".S") => "FSGNJN.S",
            ("FSGNJN", ".D") => "FSGNJN.D",
            ("FSGNJN", ".Q") => "FSGNJN.Q",
            ("FSGNJX", ".S") => "FSGNJX.S",
            ("FSGNJX", ".D") => "FSGNJX.D",
            ("FSGNJX", ".Q") => "FSGNJX.Q",
            ("FMIN", ".S") => "FMIN.S",
            ("FMIN", ".D") => "FMIN.D",
            ("FMIN", ".Q") => "FMIN.Q",
            ("FMAX", ".S") => "FMAX.S",
            ("FMAX", ".D") => "FMAX.D",
            ("FMAX", ".Q") => "FMAX.Q",
            ("FLE", ".S") => "FLE.S",
            ("FLE", ".D") => "FLE.D",
            ("FLE", ".Q") => "FLE.Q",
            ("FLT", ".S") => "FLT.S",
            ("FLT", ".D") => "FLT.D",
            ("FLT", ".Q") => "FLT.Q",
            ("FEQ", ".S") => "FEQ.S",
            ("FEQ", ".D") => "FEQ.D",
            ("FEQ", ".Q") => "FEQ.Q",
            ("FCLASS", ".S") => "FCLASS.S",
            ("FCLASS", ".D") => "FCLASS.D",
            ("FCLASS", ".Q") => "FCLASS.Q",
            ("FMV.X", ".S") => "FMV.X.S",
            ("FMV.X", ".D") => "FMV.X.D",
            ("FMV.X", ".Q") => "FMV.X.Q",
            ("FMV", ".S") => "FMV.S.X",
            ("FMV", ".D") => "FMV.D.X",
            ("FMV", ".Q") => "FMV.Q.X",
            _ => base,
        }
    }

    /// Gets the name for FP-to-integer conversion instructions.
    ///
    /// ---
    ///
    /// 获取浮点到整数转换指令的名称。
    fn get_fcvt_int_name(_src_fmt: u8, _dst_fmt: u8, rs2: u8) -> &'static str {
        // rs2 encodes the conversion type
        match rs2 {
            // To signed 32-bit
            0x00 => "FCVT.W.S",
            0x01 => "FCVT.W.D",
            0x03 => "FCVT.W.Q",
            // To unsigned 32-bit
            0x04 => "FCVT.WU.S",
            0x05 => "FCVT.WU.D",
            0x07 => "FCVT.WU.Q",
            // To signed 64-bit
            0x08 => "FCVT.L.S",
            0x09 => "FCVT.L.D",
            0x0b => "FCVT.L.Q",
            // To unsigned 64-bit
            0x0c => "FCVT.LU.S",
            0x0d => "FCVT.LU.D",
            0x0f => "FCVT.LU.Q",
            // From signed 32-bit
            0x10 => "FCVT.S.W",
            0x11 => "FCVT.D.W",
            0x13 => "FCVT.Q.W",
            // From unsigned 32-bit
            0x14 => "FCVT.S.WU",
            0x15 => "FCVT.D.WU",
            0x17 => "FCVT.Q.WU",
            // From signed 64-bit
            0x18 => "FCVT.S.L",
            0x19 => "FCVT.D.L",
            0x1b => "FCVT.Q.L",
            // From unsigned 64-bit
            0x1c => "FCVT.S.LU",
            0x1d => "FCVT.D.LU",
            0x1f => "FCVT.Q.LU",
            // From/To signed 128-bit
            0x20 => "FCVT.S.Q",
            0x21 => "FCVT.D.Q",
            0x22 => "FCVT.Q.S",
            0x23 => "FCVT.Q.D",
            _ => "FCVT_UNKNOWN",
        }
    }

    /// Gets the name for FP-to-FP conversion instructions.
    ///
    /// ---
    ///
    /// 获取浮点到浮点转换指令的名称。
    fn get_fcvt_fp_name(fmt: u8, rs2: u8) -> &'static str {
        // rs2 encodes the source format for FCVT.S.D, FCVT.D.S, etc.
        match (fmt, rs2) {
            // S -> D
            (0, 1) => "FCVT.D.S",
            // D -> S
            (1, 0) => "FCVT.S.D",
            // S -> Q
            (0, 3) => "FCVT.Q.S",
            // Q -> S
            (3, 0) => "FCVT.S.Q",
            // D -> Q
            (1, 3) => "FCVT.Q.D",
            // Q -> D
            (3, 1) => "FCVT.D.Q",
            _ => "FCVT_FP_UNKNOWN",
        }
    }
}
