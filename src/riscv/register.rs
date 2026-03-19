//! Register Module
//!
//! Implements the 128-bit register file for RISC-V.
//! Contains 32 general-purpose registers (x0-x31), 32 floating-point
//! registers (f0-f31), and special registers (PC, FCSR).
//!
//! # General-Purpose Register ABI Names
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
//! # Floating-Point Register ABI Names
//!
//! | Register | ABI Name | Description |
//! |----------|----------|-------------|
//! | f0-f7 | ft0-ft7 | FP temporaries |
//! | f8-f9 | fs0-fs1 | FP saved registers |
//! | f10-f17 | fa0-fa7 | FP arguments / Return values |
//! | f18-f27 | fs2-fs11 | FP saved registers |
//! | f28-f31 | ft8-ft11 | FP temporaries |
//!
//! ---
//!
//! 寄存器模块
//!
//! 实现 RISC-V 的 128 位寄存器组。
//! 包含 32 个通用寄存器 (x0-x31)、32 个浮点寄存器 (f0-f31)
//! 和特殊寄存器 (PC, FCSR)。
//!
//! # 通用寄存器 ABI 名称
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
//!
//! # 浮点寄存器 ABI 名称
//!
//! | 寄存器 | ABI 名称 | 描述 |
//! |--------|----------|------|
//! | f0-f7 | ft0-ft7 | 浮点临时寄存器 |
//! | f8-f9 | fs0-fs1 | 浮点保存寄存器 |
//! | f10-f17 | fa0-fa7 | 浮点参数 / 返回值 |
//! | f18-f27 | fs2-fs11 | 浮点保存寄存器 |
//! | f28-f31 | ft8-ft11 | 浮点临时寄存器 |

#![allow(dead_code)]

type Register128 = u128;
type Register64 = u64;
type Register32 = u32;

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

/// Floating-point register index enumeration.
///
/// Represents the 32 floating-point registers.
///
/// ---
///
/// 浮点寄存器索引枚举。
///
/// 表示 32 个浮点寄存器。
#[repr(u8)]
#[derive(PartialOrd, PartialEq, Clone, Copy, Debug)]
pub enum FRegisterIndex {
    F0 = 0,     // ft0 (temporary)
    F1 = 1,     // ft1
    F2 = 2,     // ft2
    F3 = 3,     // ft3
    F4 = 4,     // ft4
    F5 = 5,     // ft5
    F6 = 6,     // ft6
    F7 = 7,     // ft7
    F8 = 8,     // fs0 (saved)
    F9 = 9,     // fs1
    F10 = 10,   // fa0 (argument/return)
    F11 = 11,   // fa1
    F12 = 12,   // fa2
    F13 = 13,   // fa3
    F14 = 14,   // fa4
    F15 = 15,   // fa5
    F16 = 16,   // fa6
    F17 = 17,   // fa7
    F18 = 18,   // fs2 (saved)
    F19 = 19,   // fs3
    F20 = 20,   // fs4
    F21 = 21,   // fs5
    F22 = 22,   // fs6
    F23 = 23,   // fs7
    F24 = 24,   // fs8
    F25 = 25,   // fs9
    F26 = 26,   // fs10
    F27 = 27,   // fs11
    F28 = 28,   // ft8 (temporary)
    F29 = 29,   // ft9
    F30 = 30,   // ft10
    F31 = 31,   // ft11
}

/// Rounding mode for floating-point operations.
///
/// ---
///
/// 浮点运算舍入模式。
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RoundingMode {
    /// Round to Nearest, ties to Even (向最近偶数舍入)
    Rne = 0,
    /// Round toward Negative infinity (向负无穷舍入)
    Rtz = 1,
    /// Round toward Zero (向零舍入)
    Rdn = 2,
    /// Round toward Positive infinity (向正无穷舍入)
    Rup = 3,
    /// Round to Nearest, ties to Max Magnitude (向最大幅值舍入)
    Rmm = 4,
    /// Dynamic rounding mode (use RM field in FCSR) (动态舍入模式)
    Dyn = 7,
}

impl RoundingMode {
    /// Creates a RoundingMode from a u8 value.
    ///
    /// ---
    ///
    /// 从 u8 值创建 RoundingMode。
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => RoundingMode::Rne,
            1 => RoundingMode::Rtz,
            2 => RoundingMode::Rdn,
            3 => RoundingMode::Rup,
            4 => RoundingMode::Rmm,
            7 => RoundingMode::Dyn,
            _ => RoundingMode::Rne, // Default to RNE
        }
    }
}

/// Floating-point exception flags.
///
/// These flags indicate exceptional conditions during FP operations.
///
/// ---
///
/// 浮点异常标志。
///
/// 这些标志表示浮点操作期间的异常条件。
#[derive(Clone, Copy, Debug, Default)]
pub struct FpExceptionFlags {
    /// Invalid operation (无效操作)
    pub nv: bool,
    /// Divide by zero (除以零)
    pub dz: bool,
    /// Overflow (溢出)
    pub of: bool,
    /// Underflow (下溢)
    pub uf: bool,
    /// Inexact (不精确)
    pub nx: bool,
}

impl FpExceptionFlags {
    /// Creates new exception flags all set to false.
    ///
    /// ---
    ///
    /// 创建所有标志为 false 的异常标志。
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if any flag is set.
    ///
    /// ---
    ///
    /// 如果任何标志被设置则返回 true。
    pub fn any(&self) -> bool {
        self.nv || self.dz || self.of || self.uf || self.nx
    }

    /// Converts flags to a 5-bit value.
    ///
    /// Bits: NV(4), DZ(3), OF(2), UF(1), NX(0)
    ///
    /// ---
    ///
    /// 将标志转换为 5 位值。
    ///
    /// 位：NV(4), DZ(3), OF(2), UF(1), NX(0)
    pub fn to_bits(&self) -> u8 {
        let mut bits = 0u8;
        if self.nv { bits |= 0x10; }
        if self.dz { bits |= 0x08; }
        if self.of { bits |= 0x04; }
        if self.uf { bits |= 0x02; }
        if self.nx { bits |= 0x01; }
        bits
    }

    /// Creates flags from a 5-bit value.
    ///
    /// ---
    ///
    /// 从 5 位值创建标志。
    pub fn from_bits(bits: u8) -> Self {
        Self {
            nv: (bits & 0x10) != 0,
            dz: (bits & 0x08) != 0,
            of: (bits & 0x04) != 0,
            uf: (bits & 0x02) != 0,
            nx: (bits & 0x01) != 0,
        }
    }
}

/// Floating-Point Control and Status Register (FCSR).
///
/// Contains the rounding mode and exception flags.
///
/// Bit layout:
/// - Bits 7:5 - Reserved (should be 0)
/// - Bits 4:0 - Exception flags (NV, DZ, OF, UF, NX)
/// - Bits 23:8 - Reserved for D extension
/// - Bits 31:24 - Reserved
///
/// For fcsr register access:
/// - fcsr[7:5] = Reserved
/// - fcsr[4:0] = fflags (exception flags)
/// - fcsr[7:5] + fcsr[4:0] can be accessed via fflags and frm separately
///
/// ---
///
/// 浮点控制状态寄存器 (FCSR)。
///
/// 包含舍入模式和异常标志。
///
/// 位布局：
/// - 位 7:5 - 保留（应为 0）
/// - 位 4:0 - 异常标志 (NV, DZ, OF, UF, NX)
/// - 位 23:8 - D 扩展保留
/// - 位 31:24 - 保留
#[derive(Clone, Copy, Debug)]
pub struct FCSR {
    /// Rounding mode (舍入模式)
    pub frm: RoundingMode,
    /// Exception flags (异常标志)
    pub fflags: FpExceptionFlags,
}

impl Default for FCSR {
    fn default() -> Self {
        Self::new()
    }
}

impl FCSR {
    /// Creates a new FCSR with default values.
    ///
    /// Default: RNE rounding, no exceptions.
    ///
    /// ---
    ///
    /// 使用默认值创建新 FCSR。
    ///
    /// 默认：RNE 舍入，无异常。
    pub fn new() -> Self {
        Self {
            frm: RoundingMode::Rne,
            fflags: FpExceptionFlags::new(),
        }
    }

    /// Returns the FCSR as a 32-bit value.
    ///
    /// Bits [4:0] = fflags, Bits [7:5] = frm
    ///
    /// ---
    ///
    /// 将 FCSR 作为 32 位值返回。
    ///
    /// 位 [4:0] = fflags, 位 [7:5] = frm
    pub fn to_u32(&self) -> Register32 {
        let frm_bits = self.frm as u8 as Register32;
        let fflags_bits = self.fflags.to_bits() as Register32;
        (frm_bits << 5) | fflags_bits
    }

    /// Sets FCSR from a 32-bit value.
    ///
    /// ---
    ///
    /// 从 32 位值设置 FCSR。
    pub fn from_u32(&mut self, value: Register32) {
        self.frm = RoundingMode::from_u8(((value >> 5) & 0x7) as u8);
        self.fflags = FpExceptionFlags::from_bits((value & 0x1f) as u8);
    }

    /// Returns the fflags field (bits [4:0]).
    ///
    /// ---
    ///
    /// 返回 fflags 字段（位 [4:0]）。
    pub fn get_fflags(&self) -> Register32 {
        self.fflags.to_bits() as Register32
    }

    /// Sets the fflags field.
    ///
    /// ---
    ///
    /// 设置 fflags 字段。
    pub fn set_fflags(&mut self, value: Register32) {
        self.fflags = FpExceptionFlags::from_bits((value & 0x1f) as u8);
    }

    /// Returns the frm field (bits [7:5]).
    ///
    /// ---
    ///
    /// 返回 frm 字段（位 [7:5]）。
    pub fn get_frm(&self) -> Register32 {
        self.frm as u8 as Register32
    }

    /// Sets the frm field.
    ///
    /// ---
    ///
    /// 设置 frm 字段。
    pub fn set_frm(&mut self, value: Register32) {
        self.frm = RoundingMode::from_u8((value & 0x7) as u8);
    }

    /// Clears all exception flags.
    ///
    /// ---
    ///
    /// 清除所有异常标志。
    pub fn clear_flags(&mut self) {
        self.fflags = FpExceptionFlags::new();
    }
}

// ========================================
// V Extension: Vector Registers / V 扩展：向量寄存器
// ========================================

/// Vector register index enumeration.
///
/// Represents the 32 vector registers (v0-v31).
///
/// ---
///
/// 向量寄存器索引枚举。
///
/// 表示 32 个向量寄存器 (v0-v31)。
#[repr(u8)]
#[derive(PartialOrd, PartialEq, Clone, Copy)]
pub enum VRegisterIndex {
    V0 = 0,
    V1 = 1,
    V2 = 2,
    V3 = 3,
    V4 = 4,
    V5 = 5,
    V6 = 6,
    V7 = 7,
    V8 = 8,
    V9 = 9,
    V10 = 10,
    V11 = 11,
    V12 = 12,
    V13 = 13,
    V14 = 14,
    V15 = 15,
    V16 = 16,
    V17 = 17,
    V18 = 18,
    V19 = 19,
    V20 = 20,
    V21 = 21,
    V22 = 22,
    V23 = 23,
    V24 = 24,
    V25 = 25,
    V26 = 26,
    V27 = 27,
    V28 = 28,
    V29 = 29,
    V30 = 30,
    V31 = 31,
}

/// Vector element width (SEW) selection.
///
/// Encoded in vtype field.
///
/// ---
///
/// 向量元素宽度 (SEW) 选择。
///
/// 编码在 vtype 字段中。
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VSelemWidth {
    /// 8-bit elements
    E8 = 0b000,
    /// 16-bit elements
    E16 = 0b001,
    /// 32-bit elements
    E32 = 0b010,
    /// 64-bit elements
    E64 = 0b011,
    /// 128-bit elements (RV128)
    E128 = 0b100,
    /// 256-bit elements (RV128)
    E256 = 0b101,
    /// 512-bit elements (RV128)
    E512 = 0b110,
    /// 1024-bit elements (RV128)
    E1024 = 0b111,
}

impl VSelemWidth {
    /// Returns the element width in bits.
    ///
    /// ---
    ///
    /// 返回元素宽度（位）。
    pub fn bits(&self) -> u32 {
        match self {
            VSelemWidth::E8 => 8,
            VSelemWidth::E16 => 16,
            VSelemWidth::E32 => 32,
            VSelemWidth::E64 => 64,
            VSelemWidth::E128 => 128,
            VSelemWidth::E256 => 256,
            VSelemWidth::E512 => 512,
            VSelemWidth::E1024 => 1024,
        }
    }

    /// Returns the element width in bytes.
    ///
    /// ---
    ///
    /// 返回元素宽度（字节）。
    pub fn bytes(&self) -> u32 {
        self.bits() / 8
    }

    /// Creates VSelemWidth from encoding bits.
    ///
    /// ---
    ///
    /// 从编码位创建 VSelemWidth。
    pub fn from_bits(bits: u8) -> Self {
        match bits & 0x7 {
            0b000 => VSelemWidth::E8,
            0b001 => VSelemWidth::E16,
            0b010 => VSelemWidth::E32,
            0b011 => VSelemWidth::E64,
            0b100 => VSelemWidth::E128,
            0b101 => VSelemWidth::E256,
            0b110 => VSelemWidth::E512,
            0b111 => VSelemWidth::E1024,
            _ => VSelemWidth::E8,
        }
    }
}

/// Vector multiplier for LMUL (Length Multiplier).
///
/// LMUL can be fractional (1/8, 1/4, 1/2) or integer (1, 2, 4, 8, ...).
///
/// ---
///
/// LMUL（长度乘数）的向量乘数。
///
/// LMUL 可以是分数（1/8, 1/4, 1/2）或整数（1, 2, 4, 8, ...）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VLmul {
    /// LMUL = 1/8
    LmulF8 = 0b000,
    /// LMUL = 1/4
    LmulF4 = 0b001,
    /// LMUL = 1/2
    LmulF2 = 0b010,
    /// LMUL = 1
    Lmul1 = 0b011,
    /// LMUL = 2
    Lmul2 = 0b100,
    /// LMUL = 4
    Lmul4 = 0b101,
    /// LMUL = 8
    Lmul8 = 0b110,
    /// LMUL = 16 (RV128)
    Lmul16 = 0b111,
}

impl VLmul {
    /// Returns the LMUL multiplier as a floating-point value.
    ///
    /// ---
    ///
    /// 返回 LMUL 乘数的浮点值。
    pub fn multiplier(&self) -> f32 {
        match self {
            VLmul::LmulF8 => 0.125,
            VLmul::LmulF4 => 0.25,
            VLmul::LmulF2 => 0.5,
            VLmul::Lmul1 => 1.0,
            VLmul::Lmul2 => 2.0,
            VLmul::Lmul4 => 4.0,
            VLmul::Lmul8 => 8.0,
            VLmul::Lmul16 => 16.0,
        }
    }

    /// Creates VLmul from encoding bits.
    ///
    /// ---
    ///
    /// 从编码位创建 VLmul。
    pub fn from_bits(bits: u8) -> Self {
        match bits & 0x7 {
            0b000 => VLmul::LmulF8,
            0b001 => VLmul::LmulF4,
            0b010 => VLmul::LmulF2,
            0b011 => VLmul::Lmul1,
            0b100 => VLmul::Lmul2,
            0b101 => VLmul::Lmul4,
            0b110 => VLmul::Lmul8,
            0b111 => VLmul::Lmul16,
            _ => VLmul::Lmul1,
        }
    }
}

/// Vector type register (vtype).
///
/// Controls the configuration of vector operations.
///
/// ---
///
/// 向量类型寄存器 (vtype)。
///
/// 控制向量操作的配置。
#[derive(Clone, Copy, Debug)]
pub struct VType {
    /// Vector element width (SEW) - bits [2:0]
    /// 向量元素宽度 (SEW) - 位 [2:0]
    pub vsew: VSelemWidth,
    /// Vector length multiplier (LMUL) - bits [5:3]
    /// 向量长度乘数 (LMUL) - 位 [5:3]
    pub vlmul: VLmul,
    /// Vector tail agnostic - bit [6]
    /// 向量尾部不可知 - 位 [6]
    pub vta: bool,
    /// Vector mask agnostic - bit [7]
    /// 向量掩码不可知 - 位 [7]
    pub vma: bool,
}

impl VType {
    /// Creates a new VType with default values.
    ///
    /// ---
    ///
    /// 创建具有默认值的新 VType。
    pub fn new() -> Self {
        Self {
            vsew: VSelemWidth::E8,
            vlmul: VLmul::Lmul1,
            vta: false,
            vma: false,
        }
    }

    /// Converts VType to a 64-bit value.
    ///
    /// ---
    ///
    /// 将 VType 转换为 64 位值。
    pub fn to_u64(&self) -> u64 {
        let mut value: u64 = 0;
        value |= (self.vsew.clone() as u64) & 0x7;
        value |= ((self.vlmul.clone() as u64) & 0x7) << 3;
        value |= (if self.vta { 1 } else { 0 }) << 6;
        value |= (if self.vma { 1 } else { 0 }) << 7;
        value
    }

    /// Creates VType from a 64-bit value.
    ///
    /// ---
    ///
    /// 从 64 位值创建 VType。
    pub fn from_u64(value: u64) -> Self {
        Self {
            vsew: VSelemWidth::from_bits((value & 0x7) as u8),
            vlmul: VLmul::from_bits(((value >> 3) & 0x7) as u8),
            vta: (value >> 6) & 1 != 0,
            vma: (value >> 7) & 1 != 0,
        }
    }
}

impl Default for VType {
    fn default() -> Self {
        Self::new()
    }
}

/// Vector machine state (V-extension CSRs).
///
/// ---
///
/// 向量机器状态（V 扩展 CSR）。
#[derive(Clone, Debug)]
pub struct VectorState {
    /// Vector length (VL) - number of elements to process
    /// 向量长度 (VL) - 要处理的元素数量
    pub vl: u64,
    /// Vector type (VTYPE) - vector configuration
    /// 向量类型 (VTYPE) - 向量配置
    pub vtype: VType,
    /// Vector start index (VSTART) - element index to start from
    /// 向量起始索引 (VSTART) - 起始元素索引
    pub vstart: u64,
    /// Vector length in bits (VLEN) - implementation parameter
    /// 向量位长度 (VLEN) - 实现参数
    pub vlen: u64,
    /// Vector mask register state
    /// 向量掩码寄存器状态
    pub vlenb: u64,
}

impl VectorState {
    /// Creates a new VectorState with default values.
    /// VLEN is set to 128 for RV128V.
    ///
    /// ---
    ///
    /// 创建具有默认值的新 VectorState。
    /// VLEN 设置为 128 用于 RV128V。
    pub fn new() -> Self {
        Self {
            vl: 0,
            vtype: VType::new(),
            vstart: 0,
            vlen: 128,
            vlenb: 16, // 128 / 8 = 16 bytes
        }
    }

    /// Calculates the maximum vector length (VLMAX) for current configuration.
    ///
    /// VLMAX = (VLEN * LMUL) / SEW
    ///
    /// ---
    ///
    /// 计算当前配置的最大向量长度 (VLMAX)。
    ///
    /// VLMAX = (VLEN * LMUL) / SEW
    pub fn vlmax(&self) -> u64 {
        let lmul = self.vtype.vlmul.multiplier();
        let sew = self.vtype.vsew.bits() as f32;
        ((self.vlen as f32 * lmul) / sew) as u64
    }

    /// Sets VL based on application request (vsetvli instruction).
    ///
    /// ---
    ///
    /// 根据应用请求设置 VL（vsetvli 指令）。
    pub fn set_vl(&mut self, requested_vl: u64) {
        let vlmax = self.vlmax();
        self.vl = if requested_vl <= vlmax { requested_vl } else { vlmax };
        self.vstart = 0;
    }

    /// Resets the vector state.
    ///
    /// ---
    ///
    /// 重置向量状态。
    pub fn reset(&mut self) {
        self.vl = 0;
        self.vtype = VType::new();
        self.vstart = 0;
    }
}

impl Default for VectorState {
    fn default() -> Self {
        Self::new()
    }
}

/// Special registers (PC, FCSR, Vector, CSR).
///
/// ---
///
/// 特殊寄存器 (PC, FCSR, 向量, CSR)。
pub struct SpecialRegisters {
    /// Program counter (程序计数器)
    pc: Register128,
    /// Floating-point control and status register (浮点控制状态寄存器)
    fcsr: FCSR,
    /// Vector state (向量状态)
    vector: VectorState,
    /// Control and Status Registers (控制状态寄存器)
    csr: CsrFile,
}

/// Register file with 32 128-bit general-purpose registers and 32 128-bit floating-point registers.
///
/// x0 is hardwired to zero - writes are silently ignored.
///
/// ---
///
/// 包含 32 个 128 位通用寄存器和 32 个 128 位浮点寄存器的寄存器组。
///
/// x0 硬连线为零 - 写入被静默忽略。
pub struct Register {
    /// General-purpose registers (通用寄存器)
    registers: [Register128; 32],
    /// Floating-point registers (浮点寄存器)
    fp_registers: [Register128; 32],
    /// Vector registers (向量寄存器) - each can hold multiple elements
    v_registers: [Register128; 32],
    /// Special registers (特殊寄存器)
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
            fp_registers: [0; 32],
            v_registers: [0; 32],
            special: SpecialRegisters {
                pc: 0,
                fcsr: FCSR::new(),
                vector: VectorState::new(),
                csr: CsrFile::new(),
            },
        };
        tmp.reset();
        tmp
    }

    /// Reads a general-purpose register value.
    ///
    /// Reading x0 always returns 0.
    ///
    /// ---
    ///
    /// 读取通用寄存器值。
    ///
    /// 读取 x0 始终返回 0。
    pub fn read(&self, index: RegisterIndex) -> Register128 {
        if index > RegisterIndex::X31 {
            panic!("Invalid register index: Index out of range");
        }
        self.registers[index as usize]
    }

    /// Writes a value to a general-purpose register.
    ///
    /// Writes to x0 are ignored (remains zero).
    ///
    /// ---
    ///
    /// 将值写入通用寄存器。
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

    /// Reads a floating-point register value.
    ///
    /// ---
    ///
    /// 读取浮点寄存器值。
    pub fn read_fp(&self, index: FRegisterIndex) -> Register128 {
        if index > FRegisterIndex::F31 {
            panic!("Invalid floating-point register index: Index out of range");
        }
        self.fp_registers[index as usize]
    }

    /// Writes a value to a floating-point register.
    ///
    /// ---
    ///
    /// 将值写入浮点寄存器。
    pub fn write_fp(&mut self, index: FRegisterIndex, value: Register128) {
        if index > FRegisterIndex::F31 {
            panic!("Invalid floating-point register index: Index out of range");
        }
        self.fp_registers[index as usize] = value;
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

    /// Returns the FCSR value.
    ///
    /// ---
    ///
    /// 返回 FCSR 值。
    pub fn get_fcsr(&self) -> Register32 {
        self.special.fcsr.to_u32()
    }

    /// Sets the FCSR value.
    ///
    /// ---
    ///
    /// 设置 FCSR 值。
    pub fn set_fcsr(&mut self, value: Register32) {
        self.special.fcsr.from_u32(value);
    }

    /// Returns a reference to the FCSR.
    ///
    /// ---
    ///
    /// 返回 FCSR 的引用。
    pub fn get_fcsr_ref(&self) -> &FCSR {
        &self.special.fcsr
    }

    /// Returns a mutable reference to the FCSR.
    ///
    /// ---
    ///
    /// 返回 FCSR 的可变引用。
    pub fn get_fcsr_mut(&mut self) -> &mut FCSR {
        &mut self.special.fcsr
    }

    /// Returns the current rounding mode.
    ///
    /// ---
    ///
    /// 返回当前舍入模式。
    pub fn get_rounding_mode(&self) -> RoundingMode {
        self.special.fcsr.frm
    }

    /// Sets the rounding mode.
    ///
    /// ---
    ///
    /// 设置舍入模式。
    pub fn set_rounding_mode(&mut self, rm: RoundingMode) {
        self.special.fcsr.frm = rm;
    }

    // ========================================
    // Vector Register Methods / 向量寄存器方法
    // ========================================

    /// Reads a vector register value.
    ///
    /// ---
    ///
    /// 读取向量寄存器值。
    pub fn read_v(&self, index: VRegisterIndex) -> Register128 {
        if index > VRegisterIndex::V31 {
            panic!("Invalid vector register index: Index out of range");
        }
        self.v_registers[index as usize]
    }

    /// Writes a value to a vector register.
    ///
    /// ---
    ///
    /// 将值写入向量寄存器。
    pub fn write_v(&mut self, index: VRegisterIndex, value: Register128) {
        if index > VRegisterIndex::V31 {
            panic!("Invalid vector register index: Index out of range");
        }
        self.v_registers[index as usize] = value;
    }

    /// Returns the vector state.
    ///
    /// ---
    ///
    /// 返回向量状态。
    pub fn get_vector_state(&self) -> &VectorState {
        &self.special.vector
    }

    /// Returns a mutable reference to the vector state.
    ///
    /// ---
    ///
    /// 返回向量状态的可变引用。
    pub fn get_vector_state_mut(&mut self) -> &mut VectorState {
        &mut self.special.vector
    }

    /// Returns VL (Vector Length).
    ///
    /// ---
    ///
    /// 返回 VL（向量长度）。
    pub fn get_vl(&self) -> u64 {
        self.special.vector.vl
    }

    /// Sets VL (Vector Length).
    ///
    /// ---
    ///
    /// 设置 VL（向量长度）。
    pub fn set_vl(&mut self, vl: u64) {
        self.special.vector.set_vl(vl);
    }

    /// Returns VSTART (Vector Start Index).
    ///
    /// ---
    ///
    /// 返回 VSTART（向量起始索引）。
    pub fn get_vstart(&self) -> u64 {
        self.special.vector.vstart
    }

    /// Sets VSTART (Vector Start Index).
    ///
    /// ---
    ///
    /// 设置 VSTART（向量起始索引）。
    pub fn set_vstart(&mut self, vstart: u64) {
        self.special.vector.vstart = vstart;
    }

    /// Returns VTYPE as u64.
    ///
    /// ---
    ///
    /// 返回 VTYPE 作为 u64。
    pub fn get_vtype(&self) -> u64 {
        self.special.vector.vtype.to_u64()
    }

    /// Sets VTYPE from u64.
    ///
    /// ---
    ///
    /// 从 u64 设置 VTYPE。
    pub fn set_vtype(&mut self, vtype: u64) {
        self.special.vector.vtype = VType::from_u64(vtype);
    }

    // ========================================
    // CSR Access Methods / CSR 访问方法
    // ========================================

    /// Reads a CSR by address.
    ///
    /// ---
    ///
    /// 按地址读取 CSR。
    pub fn read_csr(&self, addr: CsrAddress) -> Register128 {
        // Handle special CSRs that are backed by other structures
        match addr {
            CsrAddress::Fflags => self.special.fcsr.get_fflags() as Register128,
            CsrAddress::Frm => self.special.fcsr.get_frm() as Register128,
            CsrAddress::Fcsr => self.special.fcsr.to_u32() as Register128,
            CsrAddress::Vl => self.special.vector.vl as Register128,
            CsrAddress::Vtype => self.special.vector.vtype.to_u64() as Register128,
            CsrAddress::Vlenb => self.special.vector.vlenb as Register128,
            CsrAddress::Vstart => self.special.vector.vstart as Register128,
            _ => self.special.csr.read(addr),
        }
    }

    /// Writes a CSR by address.
    ///
    /// ---
    ///
    /// 按地址写入 CSR。
    pub fn write_csr(&mut self, addr: CsrAddress, value: Register128) {
        // Handle special CSRs that are backed by other structures
        match addr {
            CsrAddress::Fflags => self.special.fcsr.set_fflags(value as Register32),
            CsrAddress::Frm => self.special.fcsr.set_frm(value as Register32),
            CsrAddress::Fcsr => self.special.fcsr.from_u32(value as Register32),
            CsrAddress::Vl => self.special.vector.vl = value as u64,
            CsrAddress::Vtype => self.special.vector.vtype = VType::from_u64(value as u64),
            CsrAddress::Vstart => self.special.vector.vstart = value as u64,
            _ => self.special.csr.write(addr, value),
        }
    }

    /// Returns a reference to the CSR file.
    ///
    /// ---
    ///
    /// 返回 CSR 文件的引用。
    pub fn get_csr_file(&self) -> &CsrFile {
        &self.special.csr
    }

    /// Returns a mutable reference to the CSR file.
    ///
    /// ---
    ///
    /// 返回 CSR 文件的可变引用。
    pub fn get_csr_file_mut(&mut self) -> &mut CsrFile {
        &mut self.special.csr
    }

    /// Increments the cycle counter.
    ///
    /// ---
    ///
    /// 递增周期计数器。
    pub fn increment_cycle(&mut self) {
        self.special.csr.increment_cycle();
    }

    /// Increments the instruction retired counter.
    ///
    /// ---
    ///
    /// 递增指令完成计数器。
    pub fn increment_instret(&mut self) {
        self.special.csr.increment_instret();
    }

    /// Resets all registers to zero.
    ///
    /// ---
    ///
    /// 将所有寄存器重置为零。
    pub fn reset(&mut self) {
        self.registers = [0; 32];
        self.fp_registers = [0; 32];
        self.v_registers = [0; 32];
        self.special = SpecialRegisters {
            pc: 0,
            fcsr: FCSR::new(),
            vector: VectorState::new(),
            csr: CsrFile::new(),
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

impl Default for Register {
    fn default() -> Self {
        Self::new()
    }
}

// ========================================
// Zicsr Extension: Control and Status Registers / Zicsr 扩展：控制状态寄存器
// ========================================

/// CSR (Control and Status Register) address definitions.
///
/// CSR addresses are 12-bit values organized by privilege level:
/// - 0x000-0x1FF: Unprivileged/User-level CSRs
/// - 0x200-0x5FF: Supervisor-level CSRs
/// - 0x600-0x7FF: Hypervisor-level CSRs (not implemented)
/// - 0x800-0xBFF: Machine-level CSRs
/// - 0xC00-0xFFF: Debug/Trace CSRs
///
/// ---
///
/// CSR（控制状态寄存器）地址定义。
///
/// CSR 地址是 12 位值，按特权级别组织：
/// - 0x000-0x1FF：非特权/用户级 CSR
/// - 0x200-0x5FF：监管者级 CSR
/// - 0x600-0x7FF：虚拟化管理级 CSR（未实现）
/// - 0x800-0xBFF：机器级 CSR
/// - 0xC00-0xFFF：调试/追踪 CSR
#[repr(u16)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CsrAddress {
    // ========================================
    // User-level CSRs / 用户级 CSR
    // ========================================
    /// Floating-Point Accrued Exceptions (浮点累积异常)
    Fflags = 0x001,
    /// Floating-Point Dynamic Rounding Mode (浮点动态舍入模式)
    Frm = 0x002,
    /// Floating-Point Control and Status Register (浮点控制状态寄存器)
    Fcsr = 0x003,
    /// Vector Start Index (向量起始索引)
    Vstart = 0x008,
    /// Vector Register Settings (向量寄存器设置)
    Vxsat = 0x009,
    /// Vector Fixed-Point Saturate Flag (向量定点饱和标志)
    Vxrm = 0x00A,
    /// Vector Control and Status Register (向量控制状态寄存器)
    Vcsr = 0x00F,
    /// Vector Length (向量长度)
    Vl = 0xC20,
    /// Vector Type (向量类型)
    Vtype = 0xC21,
    /// Vector Byte Length (向量字节长度)
    Vlenb = 0xC22,

    // ========================================
    // Machine-level CSRs / 机器级 CSR
    // ========================================
    /// Machine Status Register (机器状态寄存器)
    Mstatus = 0x300,
    /// Machine ISA Register (机器 ISA 寄存器)
    Misa = 0x301,
    /// Machine Exception Delegation Register (机器异常委托寄存器)
    Medeleg = 0x302,
    /// Machine Interrupt Delegation Register (机器中断委托寄存器)
    Mideleg = 0x303,
    /// Machine Interrupt Enable Register (机器中断使能寄存器)
    Mie = 0x304,
    /// Machine Trap-Vector Base Address (机器陷入向量基地址)
    Mtvec = 0x305,
    /// Machine Counter Enable (机器计数器使能)
    Mcounteren = 0x306,
    /// Machine Status Register for RV128 (机器状态寄存器 RV128)
    Mstatush = 0x310,
    /// Machine Scratch Register (机器暂存寄存器)
    Mscratch = 0x340,
    /// Machine Exception Program Counter (机器异常程序计数器)
    Mepc = 0x341,
    /// Machine Trap Cause (机器陷入原因)
    Mcause = 0x342,
    /// Machine Trap Value (机器陷入值)
    Mtval = 0x343,
    /// Machine IP Register (机器 IP 寄存器)
    Mip = 0x344,
    /// Machine Virtualization Control (机器虚拟化控制)
    Menvcfg = 0x30A,
    /// Machine Virtualization Control High (机器虚拟化控制高位)
    Menvcfgh = 0x31A,
    /// Machine Physical Memory Protection Configuration (机器物理内存保护配置)
    Pmpcfg0 = 0x3A0,
    Pmpcfg1 = 0x3A1,
    Pmpcfg2 = 0x3A2,
    Pmpcfg3 = 0x3A3,
    /// Physical Memory Protection Address Registers (物理内存保护地址寄存器)
    Pmpaddr0 = 0x3B0,
    Pmpaddr1 = 0x3B1,
    Pmpaddr2 = 0x3B2,
    Pmpaddr3 = 0x3B3,
    Pmpaddr4 = 0x3B4,
    Pmpaddr5 = 0x3B5,
    Pmpaddr6 = 0x3B6,
    Pmpaddr7 = 0x3B7,
    Pmpaddr8 = 0x3B8,
    Pmpaddr9 = 0x3B9,
    Pmpaddr10 = 0x3BA,
    Pmpaddr11 = 0x3BB,
    Pmpaddr12 = 0x3BC,
    Pmpaddr13 = 0x3BD,
    Pmpaddr14 = 0x3BE,
    Pmpaddr15 = 0x3BF,

    // ========================================
    // Machine Counter/Timers / 机器计数器/定时器
    // ========================================
    /// Machine Cycle Counter (机器周期计数器)
    Mcycle = 0xB00,
    /// Machine Instructions Retired Counter (机器指令完成计数器)
    Minstret = 0xB02,
    /// Machine Cycle Counter High (机器周期计数器高位)
    Mcycleh = 0xB80,
    /// Machine Instructions Retired Counter High (机器指令完成计数器高位)
    Minstreth = 0xB82,

    // ========================================
    // Supervisor-level CSRs / 监管者级 CSR
    // ========================================
    /// Supervisor Status Register (监管者状态寄存器)
    Sstatus = 0x100,
    /// Supervisor Interrupt Enable Register (监管者中断使能寄存器)
    Sie = 0x104,
    /// Supervisor Trap-Vector Base Address (监管者陷入向量基地址)
    Stvec = 0x105,
    /// Supervisor Counter Enable (监管者计数器使能)
    Scounteren = 0x106,
    /// Supervisor Scratch Register (监管者暂存寄存器)
    Sscratch = 0x140,
    /// Supervisor Exception Program Counter (监管者异常程序计数器)
    Sepc = 0x141,
    /// Supervisor Trap Cause (监管者陷入原因)
    Scause = 0x142,
    /// Supervisor Trap Value (监管者陷入值)
    Stval = 0x143,
    /// Supervisor IP Register (监管者 IP 寄存器)
    Sip = 0x144,
    /// Supervisor Address Translation and Protection (监管者地址转换和保护)
    Satp = 0x180,
    /// Supervisor Virtualization Control (监管者虚拟化控制)
    Senvcfg = 0x10A,

    // ========================================
    // User Counter/Timers / 用户计数器/定时器
    // ========================================
    /// User Cycle Counter (用户周期计数器)
    Cycle = 0xC00,
    /// User Time Counter (用户时间计数器)
    Time = 0xC01,
    /// User Instructions Retired Counter (用户指令完成计数器)
    Instret = 0xC02,
    /// User Cycle Counter High (用户周期计数器高位)
    Cycleh = 0xC80,
    /// User Time Counter High (用户时间计数器高位)
    Timeh = 0xC81,
    /// User Instructions Retired Counter High (用户指令完成计数器高位)
    Instreth = 0xC82,

    // ========================================
    // Debug CSRs / 调试 CSR
    // ========================================
    /// Debug Control and Status (调试控制状态)
    Dcsr = 0x7B0,
    /// Debug PC (调试 PC)
    Dpc = 0x7B1,
    /// Debug Scratch Register (调试暂存寄存器)
    Dscratch0 = 0x7B2,
    Dscratch1 = 0x7B3,

    // Unknown/Invalid (未知/无效)
    Unknown = 0x000,
}

impl CsrAddress {
    /// Creates a CsrAddress from a 12-bit address value.
    ///
    /// ---
    ///
    /// 从 12 位地址值创建 CsrAddress。
    pub fn from_u16(value: u16) -> Self {
        match value {
            0x001 => CsrAddress::Fflags,
            0x002 => CsrAddress::Frm,
            0x003 => CsrAddress::Fcsr,
            0x008 => CsrAddress::Vstart,
            0x009 => CsrAddress::Vxsat,
            0x00A => CsrAddress::Vxrm,
            0x00F => CsrAddress::Vcsr,
            0xC20 => CsrAddress::Vl,
            0xC21 => CsrAddress::Vtype,
            0xC22 => CsrAddress::Vlenb,
            0x300 => CsrAddress::Mstatus,
            0x301 => CsrAddress::Misa,
            0x302 => CsrAddress::Medeleg,
            0x303 => CsrAddress::Mideleg,
            0x304 => CsrAddress::Mie,
            0x305 => CsrAddress::Mtvec,
            0x306 => CsrAddress::Mcounteren,
            0x310 => CsrAddress::Mstatush,
            0x340 => CsrAddress::Mscratch,
            0x341 => CsrAddress::Mepc,
            0x342 => CsrAddress::Mcause,
            0x343 => CsrAddress::Mtval,
            0x344 => CsrAddress::Mip,
            0x30A => CsrAddress::Menvcfg,
            0x31A => CsrAddress::Menvcfgh,
            0x3A0 => CsrAddress::Pmpcfg0,
            0x3A1 => CsrAddress::Pmpcfg1,
            0x3A2 => CsrAddress::Pmpcfg2,
            0x3A3 => CsrAddress::Pmpcfg3,
            0x3B0 => CsrAddress::Pmpaddr0,
            0x3B1 => CsrAddress::Pmpaddr1,
            0x3B2 => CsrAddress::Pmpaddr2,
            0x3B3 => CsrAddress::Pmpaddr3,
            0x3B4 => CsrAddress::Pmpaddr4,
            0x3B5 => CsrAddress::Pmpaddr5,
            0x3B6 => CsrAddress::Pmpaddr6,
            0x3B7 => CsrAddress::Pmpaddr7,
            0x3B8 => CsrAddress::Pmpaddr8,
            0x3B9 => CsrAddress::Pmpaddr9,
            0x3BA => CsrAddress::Pmpaddr10,
            0x3BB => CsrAddress::Pmpaddr11,
            0x3BC => CsrAddress::Pmpaddr12,
            0x3BD => CsrAddress::Pmpaddr13,
            0x3BE => CsrAddress::Pmpaddr14,
            0x3BF => CsrAddress::Pmpaddr15,
            0xB00 => CsrAddress::Mcycle,
            0xB02 => CsrAddress::Minstret,
            0xB80 => CsrAddress::Mcycleh,
            0xB82 => CsrAddress::Minstreth,
            0x100 => CsrAddress::Sstatus,
            0x104 => CsrAddress::Sie,
            0x105 => CsrAddress::Stvec,
            0x106 => CsrAddress::Scounteren,
            0x140 => CsrAddress::Sscratch,
            0x141 => CsrAddress::Sepc,
            0x142 => CsrAddress::Scause,
            0x143 => CsrAddress::Stval,
            0x144 => CsrAddress::Sip,
            0x180 => CsrAddress::Satp,
            0x10A => CsrAddress::Senvcfg,
            0xC00 => CsrAddress::Cycle,
            0xC01 => CsrAddress::Time,
            0xC02 => CsrAddress::Instret,
            0xC80 => CsrAddress::Cycleh,
            0xC81 => CsrAddress::Timeh,
            0xC82 => CsrAddress::Instreth,
            0x7B0 => CsrAddress::Dcsr,
            0x7B1 => CsrAddress::Dpc,
            0x7B2 => CsrAddress::Dscratch0,
            0x7B3 => CsrAddress::Dscratch1,
            _ => CsrAddress::Unknown,
        }
    }

    /// Returns the CSR name as a string.
    ///
    /// ---
    ///
    /// 返回 CSR 名称作为字符串。
    pub fn name(&self) -> &'static str {
        match self {
            CsrAddress::Fflags => "fflags",
            CsrAddress::Frm => "frm",
            CsrAddress::Fcsr => "fcsr",
            CsrAddress::Vstart => "vstart",
            CsrAddress::Vxsat => "vxsat",
            CsrAddress::Vxrm => "vxrm",
            CsrAddress::Vcsr => "vcsr",
            CsrAddress::Vl => "vl",
            CsrAddress::Vtype => "vtype",
            CsrAddress::Vlenb => "vlenb",
            CsrAddress::Mstatus => "mstatus",
            CsrAddress::Misa => "misa",
            CsrAddress::Medeleg => "medeleg",
            CsrAddress::Mideleg => "mideleg",
            CsrAddress::Mie => "mie",
            CsrAddress::Mtvec => "mtvec",
            CsrAddress::Mcounteren => "mcounteren",
            CsrAddress::Mstatush => "mstatush",
            CsrAddress::Mscratch => "mscratch",
            CsrAddress::Mepc => "mepc",
            CsrAddress::Mcause => "mcause",
            CsrAddress::Mtval => "mtval",
            CsrAddress::Mip => "mip",
            CsrAddress::Menvcfg => "menvcfg",
            CsrAddress::Menvcfgh => "menvcfgh",
            CsrAddress::Pmpcfg0 => "pmpcfg0",
            CsrAddress::Pmpcfg1 => "pmpcfg1",
            CsrAddress::Pmpcfg2 => "pmpcfg2",
            CsrAddress::Pmpcfg3 => "pmpcfg3",
            CsrAddress::Pmpaddr0 => "pmpaddr0",
            CsrAddress::Pmpaddr1 => "pmpaddr1",
            CsrAddress::Pmpaddr2 => "pmpaddr2",
            CsrAddress::Pmpaddr3 => "pmpaddr3",
            CsrAddress::Pmpaddr4 => "pmpaddr4",
            CsrAddress::Pmpaddr5 => "pmpaddr5",
            CsrAddress::Pmpaddr6 => "pmpaddr6",
            CsrAddress::Pmpaddr7 => "pmpaddr7",
            CsrAddress::Pmpaddr8 => "pmpaddr8",
            CsrAddress::Pmpaddr9 => "pmpaddr9",
            CsrAddress::Pmpaddr10 => "pmpaddr10",
            CsrAddress::Pmpaddr11 => "pmpaddr11",
            CsrAddress::Pmpaddr12 => "pmpaddr12",
            CsrAddress::Pmpaddr13 => "pmpaddr13",
            CsrAddress::Pmpaddr14 => "pmpaddr14",
            CsrAddress::Pmpaddr15 => "pmpaddr15",
            CsrAddress::Mcycle => "mcycle",
            CsrAddress::Minstret => "minstret",
            CsrAddress::Mcycleh => "mcycleh",
            CsrAddress::Minstreth => "minstreth",
            CsrAddress::Sstatus => "sstatus",
            CsrAddress::Sie => "sie",
            CsrAddress::Stvec => "stvec",
            CsrAddress::Scounteren => "scounteren",
            CsrAddress::Sscratch => "sscratch",
            CsrAddress::Sepc => "sepc",
            CsrAddress::Scause => "scause",
            CsrAddress::Stval => "stval",
            CsrAddress::Sip => "sip",
            CsrAddress::Satp => "satp",
            CsrAddress::Senvcfg => "senvcfg",
            CsrAddress::Cycle => "cycle",
            CsrAddress::Time => "time",
            CsrAddress::Instret => "instret",
            CsrAddress::Cycleh => "cycleh",
            CsrAddress::Timeh => "timeh",
            CsrAddress::Instreth => "instreth",
            CsrAddress::Dcsr => "dcsr",
            CsrAddress::Dpc => "dpc",
            CsrAddress::Dscratch0 => "dscratch0",
            CsrAddress::Dscratch1 => "dscratch1",
            CsrAddress::Unknown => "unknown",
        }
    }

    /// Returns true if this is a read-only CSR.
    /// Read-only CSRs have bits [11:10] = 0b11.
    ///
    /// ---
    ///
    /// 如果这是只读 CSR 则返回 true。
    /// 只读 CSR 的位 [11:10] = 0b11。
    pub fn is_read_only(&self) -> bool {
        let addr = *self as u16;
        (addr >> 10) == 0x3
    }
}

/// CSR register file.
///
/// Contains all Control and Status Registers.
///
/// ---
///
/// CSR 寄存器组。
///
/// 包含所有控制状态寄存器。
#[derive(Clone, Debug)]
pub struct CsrFile {
    /// Machine-level CSRs (机器级 CSR)
    pub mstatus: u128,
    pub misa: u128,
    pub medeleg: u128,
    pub mideleg: u128,
    pub mie: u128,
    pub mtvec: u128,
    pub mcounteren: u128,
    pub mstatush: u128,
    pub mscratch: u128,
    pub mepc: u128,
    pub mcause: u128,
    pub mtval: u128,
    pub mip: u128,
    pub menvcfg: u128,
    pub menvcfgh: u128,
    pub pmpcfg: [u128; 4],
    pub pmpaddr: [u128; 16],

    /// Machine counters (机器计数器)
    pub mcycle: u128,
    pub minstret: u128,

    /// Supervisor-level CSRs (监管者级 CSR)
    pub sstatus: u128,
    pub sie: u128,
    pub stvec: u128,
    pub scounteren: u128,
    pub sscratch: u128,
    pub sepc: u128,
    pub scause: u128,
    pub stval: u128,
    pub sip: u128,
    pub satp: u128,
    pub senvcfg: u128,

    /// Debug CSRs (调试 CSR)
    pub dcsr: u128,
    pub dpc: u128,
    pub dscratch: [u128; 2],

    /// Cycle counter for user mode (用户模式周期计数器)
    pub cycle: u128,
    pub time: u128,
    pub instret: u128,
}

impl CsrFile {
    /// Creates a new CSR file with default values.
    ///
    /// ---
    ///
    /// 创建具有默认值的新 CSR 文件。
    pub fn new() -> Self {
        Self {
            mstatus: 0,
            // MISA: RV128IMAFDQCV_Zicsr_Zifencei
            // XL = 128, MXL = 3 (bits [63:62] = 0b11 for RV128)
            // Extensions: IMAFDQCV_Zicsr_Zifencei
            misa: (3u128 << 62) | 0x14110D25u128,
            medeleg: 0,
            mideleg: 0,
            mie: 0,
            mtvec: 0,
            mcounteren: 0,
            mstatush: 0,
            mscratch: 0,
            mepc: 0,
            mcause: 0,
            mtval: 0,
            mip: 0,
            menvcfg: 0,
            menvcfgh: 0,
            pmpcfg: [0; 4],
            pmpaddr: [0; 16],
            mcycle: 0,
            minstret: 0,
            sstatus: 0,
            sie: 0,
            stvec: 0,
            scounteren: 0,
            sscratch: 0,
            sepc: 0,
            scause: 0,
            stval: 0,
            sip: 0,
            satp: 0,
            senvcfg: 0,
            dcsr: 0,
            dpc: 0,
            dscratch: [0; 2],
            cycle: 0,
            time: 0,
            instret: 0,
        }
    }

    /// Reads a CSR by address.
    ///
    /// ---
    ///
    /// 按地址读取 CSR。
    pub fn read(&self, addr: CsrAddress) -> u128 {
        match addr {
            CsrAddress::Fflags => 0, // Read from FCSR
            CsrAddress::Frm => 0,    // Read from FCSR
            CsrAddress::Fcsr => 0,   // Read from FCSR
            CsrAddress::Vl => 0,     // Read from VectorState
            CsrAddress::Vtype => 0,  // Read from VectorState
            CsrAddress::Vlenb => 16, // VLENB = 16 bytes for VLEN=128
            CsrAddress::Vstart => 0, // Read from VectorState
            CsrAddress::Mstatus => self.mstatus,
            CsrAddress::Misa => self.misa,
            CsrAddress::Medeleg => self.medeleg,
            CsrAddress::Mideleg => self.mideleg,
            CsrAddress::Mie => self.mie,
            CsrAddress::Mtvec => self.mtvec,
            CsrAddress::Mcounteren => self.mcounteren,
            CsrAddress::Mstatush => self.mstatush,
            CsrAddress::Mscratch => self.mscratch,
            CsrAddress::Mepc => self.mepc,
            CsrAddress::Mcause => self.mcause,
            CsrAddress::Mtval => self.mtval,
            CsrAddress::Mip => self.mip,
            CsrAddress::Menvcfg => self.menvcfg,
            CsrAddress::Menvcfgh => self.menvcfgh,
            CsrAddress::Pmpcfg0 => self.pmpcfg[0],
            CsrAddress::Pmpcfg1 => self.pmpcfg[1],
            CsrAddress::Pmpcfg2 => self.pmpcfg[2],
            CsrAddress::Pmpcfg3 => self.pmpcfg[3],
            CsrAddress::Pmpaddr0 => self.pmpaddr[0],
            CsrAddress::Pmpaddr1 => self.pmpaddr[1],
            CsrAddress::Pmpaddr2 => self.pmpaddr[2],
            CsrAddress::Pmpaddr3 => self.pmpaddr[3],
            CsrAddress::Pmpaddr4 => self.pmpaddr[4],
            CsrAddress::Pmpaddr5 => self.pmpaddr[5],
            CsrAddress::Pmpaddr6 => self.pmpaddr[6],
            CsrAddress::Pmpaddr7 => self.pmpaddr[7],
            CsrAddress::Pmpaddr8 => self.pmpaddr[8],
            CsrAddress::Pmpaddr9 => self.pmpaddr[9],
            CsrAddress::Pmpaddr10 => self.pmpaddr[10],
            CsrAddress::Pmpaddr11 => self.pmpaddr[11],
            CsrAddress::Pmpaddr12 => self.pmpaddr[12],
            CsrAddress::Pmpaddr13 => self.pmpaddr[13],
            CsrAddress::Pmpaddr14 => self.pmpaddr[14],
            CsrAddress::Pmpaddr15 => self.pmpaddr[15],
            CsrAddress::Mcycle => self.mcycle,
            CsrAddress::Minstret => self.minstret,
            CsrAddress::Mcycleh => self.mcycle >> 64,
            CsrAddress::Minstreth => self.minstret >> 64,
            CsrAddress::Sstatus => self.sstatus,
            CsrAddress::Sie => self.sie,
            CsrAddress::Stvec => self.stvec,
            CsrAddress::Scounteren => self.scounteren,
            CsrAddress::Sscratch => self.sscratch,
            CsrAddress::Sepc => self.sepc,
            CsrAddress::Scause => self.scause,
            CsrAddress::Stval => self.stval,
            CsrAddress::Sip => self.sip,
            CsrAddress::Satp => self.satp,
            CsrAddress::Senvcfg => self.senvcfg,
            CsrAddress::Cycle => self.cycle,
            CsrAddress::Time => self.time,
            CsrAddress::Instret => self.instret,
            CsrAddress::Cycleh => self.cycle >> 64,
            CsrAddress::Timeh => self.time >> 64,
            CsrAddress::Instreth => self.instret >> 64,
            CsrAddress::Dcsr => self.dcsr,
            CsrAddress::Dpc => self.dpc,
            CsrAddress::Dscratch0 => self.dscratch[0],
            CsrAddress::Dscratch1 => self.dscratch[1],
            _ => 0,
        }
    }

    /// Writes a CSR by address.
    ///
    /// ---
    ///
    /// 按地址写入 CSR。
    pub fn write(&mut self, addr: CsrAddress, value: u128) {
        match addr {
            CsrAddress::Mstatus => self.mstatus = value,
            CsrAddress::Misa => self.misa = value,
            CsrAddress::Medeleg => self.medeleg = value,
            CsrAddress::Mideleg => self.mideleg = value,
            CsrAddress::Mie => self.mie = value,
            CsrAddress::Mtvec => self.mtvec = value,
            CsrAddress::Mcounteren => self.mcounteren = value,
            CsrAddress::Mstatush => self.mstatush = value,
            CsrAddress::Mscratch => self.mscratch = value,
            CsrAddress::Mepc => self.mepc = value,
            CsrAddress::Mcause => self.mcause = value,
            CsrAddress::Mtval => self.mtval = value,
            CsrAddress::Mip => self.mip = value,
            CsrAddress::Menvcfg => self.menvcfg = value,
            CsrAddress::Menvcfgh => self.menvcfgh = value,
            CsrAddress::Pmpcfg0 => self.pmpcfg[0] = value,
            CsrAddress::Pmpcfg1 => self.pmpcfg[1] = value,
            CsrAddress::Pmpcfg2 => self.pmpcfg[2] = value,
            CsrAddress::Pmpcfg3 => self.pmpcfg[3] = value,
            CsrAddress::Pmpaddr0 => self.pmpaddr[0] = value,
            CsrAddress::Pmpaddr1 => self.pmpaddr[1] = value,
            CsrAddress::Pmpaddr2 => self.pmpaddr[2] = value,
            CsrAddress::Pmpaddr3 => self.pmpaddr[3] = value,
            CsrAddress::Pmpaddr4 => self.pmpaddr[4] = value,
            CsrAddress::Pmpaddr5 => self.pmpaddr[5] = value,
            CsrAddress::Pmpaddr6 => self.pmpaddr[6] = value,
            CsrAddress::Pmpaddr7 => self.pmpaddr[7] = value,
            CsrAddress::Pmpaddr8 => self.pmpaddr[8] = value,
            CsrAddress::Pmpaddr9 => self.pmpaddr[9] = value,
            CsrAddress::Pmpaddr10 => self.pmpaddr[10] = value,
            CsrAddress::Pmpaddr11 => self.pmpaddr[11] = value,
            CsrAddress::Pmpaddr12 => self.pmpaddr[12] = value,
            CsrAddress::Pmpaddr13 => self.pmpaddr[13] = value,
            CsrAddress::Pmpaddr14 => self.pmpaddr[14] = value,
            CsrAddress::Pmpaddr15 => self.pmpaddr[15] = value,
            CsrAddress::Mcycle => self.mcycle = value,
            CsrAddress::Minstret => self.minstret = value,
            CsrAddress::Sstatus => self.sstatus = value,
            CsrAddress::Sie => self.sie = value,
            CsrAddress::Stvec => self.stvec = value,
            CsrAddress::Scounteren => self.scounteren = value,
            CsrAddress::Sscratch => self.sscratch = value,
            CsrAddress::Sepc => self.sepc = value,
            CsrAddress::Scause => self.scause = value,
            CsrAddress::Stval => self.stval = value,
            CsrAddress::Sip => self.sip = value,
            CsrAddress::Satp => self.satp = value,
            CsrAddress::Senvcfg => self.senvcfg = value,
            CsrAddress::Cycle => self.cycle = value,
            CsrAddress::Time => self.time = value,
            CsrAddress::Instret => self.instret = value,
            CsrAddress::Dcsr => self.dcsr = value,
            CsrAddress::Dpc => self.dpc = value,
            CsrAddress::Dscratch0 => self.dscratch[0] = value,
            CsrAddress::Dscratch1 => self.dscratch[1] = value,
            _ => {} // Read-only or unknown CSRs are silently ignored
        }
    }

    /// Increments the cycle counter.
    ///
    /// ---
    ///
    /// 递增周期计数器。
    pub fn increment_cycle(&mut self) {
        self.mcycle = self.mcycle.wrapping_add(1);
        self.cycle = self.mcycle;
    }

    /// Increments the instruction retired counter.
    ///
    /// ---
    ///
    /// 递增指令完成计数器。
    pub fn increment_instret(&mut self) {
        self.minstret = self.minstret.wrapping_add(1);
        self.instret = self.minstret;
    }

    /// Resets all CSRs to default values.
    ///
    /// ---
    ///
    /// 将所有 CSR 重置为默认值。
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

impl Default for CsrFile {
    fn default() -> Self {
        Self::new()
    }
}

// ========================================
// P Extension: SIMD Types / P 扩展：SIMD 类型
// ========================================

/// SIMD element size for P extension (SIMD instructions).
///
/// ---
///
/// P 扩展的 SIMD 元素大小（SIMD 指令）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SimdElementSize {
    /// 8-bit elements (16 elements in 128-bit)
    E8 = 0,
    /// 16-bit elements (8 elements in 128-bit)
    E16 = 1,
    /// 32-bit elements (4 elements in 128-bit)
    E32 = 2,
    /// 64-bit elements (2 elements in 128-bit)
    E64 = 3,
}

impl SimdElementSize {
    /// Returns the element size in bits.
    ///
    /// ---
    ///
    /// 返回元素大小（位）。
    pub fn bits(&self) -> u32 {
        match self {
            SimdElementSize::E8 => 8,
            SimdElementSize::E16 => 16,
            SimdElementSize::E32 => 32,
            SimdElementSize::E64 => 64,
        }
    }

    /// Returns the element size in bytes.
    ///
    /// ---
    ///
    /// 返回元素大小（字节）。
    pub fn bytes(&self) -> u32 {
        self.bits() / 8
    }

    /// Returns the number of elements in a 128-bit register.
    ///
    /// ---
    ///
    /// 返回 128 位寄存器中的元素数量。
    pub fn element_count(&self) -> u32 {
        128 / self.bits()
    }
}

/// SIMD operation type for P extension.
///
/// ---
///
/// P 扩展的 SIMD 操作类型。
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SimdOp {
    /// Addition (加法)
    Add,
    /// Subtraction (减法)
    Sub,
    /// Multiplication (乘法)
    Mul,
    /// Multiplication returning high bits (乘法返回高位)
    Mulh,
    /// Multiplication returning high bits unsigned (乘法返回高位无符号)
    Mulhu,
    /// Multiplication returning high bits signed-unsigned (乘法返回高位有符号-无符号)
    Mulhsu,
    /// Division (除法)
    Div,
    /// Division unsigned (无符号除法)
    Divu,
    /// Remainder (取余)
    Rem,
    /// Remainder unsigned (无符号取余)
    Remu,
    /// And (与)
    And,
    /// Or (或)
    Or,
    /// Xor (异或)
    Xor,
    /// Shift left logical (逻辑左移)
    Sll,
    /// Shift right logical (逻辑右移)
    Srl,
    /// Shift right arithmetic (算术右移)
    Sra,
    /// Minimum (最小值)
    Min,
    /// Maximum (最大值)
    Max,
    /// Minimum unsigned (无符号最小值)
    Minu,
    /// Maximum unsigned (无符号最大值)
    Maxu,
}