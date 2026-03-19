//! RISC-V Assembler Module
//!
//! Provides assembly functionality for RISC-V source code.
//! Supports standard RISC-V instructions, pseudo-instructions,
//! labels, and various data directives.
//!
//! # Supported Features
//!
//! - All base RISC-V I-extension instructions
//! - Pseudo-instructions: nop, mv, li, la, j, ret, call, etc.
//! - Labels and symbol resolution
//! - Data directives: .byte, .word, .ascii, .space, etc.
//! - Comments: # and // style
//!
//! # Example
//!
//! ```ignore
//! use riscv::assembler::assemble_string;
//!
//! let source = r#"
//!     li a0, 42
//!     add a1, a0, a0
//!     ebreak
//! "#;
//! let machine_code = assemble_string(source, 0x0).unwrap();
//! ```
//!
//! ---
//!
//! RISC-V 汇编器模块
//!
//! 提供 RISC-V 源代码的汇编功能。
//! 支持标准 RISC-V 指令、伪指令、标签和各种数据指令。
//!
//! # 支持的功能
//!
//! - 所有基础 RISC-V I 扩展指令
//! - 伪指令：nop, mv, li, la, j, ret, call 等
//! - 标签和符号解析
//! - 数据指令：.byte, .word, .ascii, .space 等
//! - 注释：# 和 // 风格
//!
//! # 示例
//!
//! ```ignore
//! use riscv::assembler::assemble_string;
//!
//! let source = r#"
//!     li a0, 42
//!     add a1, a0, a0
//!     ebreak
//! "#;
//! let machine_code = assemble_string(source, 0x0).unwrap();
//! ```

#![allow(dead_code)]

use std::collections::HashMap;
use std::io::{self, BufRead};
use crate::riscv::register;

/// Assembly error with line number and message.
///
/// ---
///
/// 汇编错误，包含行号和消息。
#[derive(Debug, Clone)]
pub struct AssemblyError {
    /// Line number where the error occurred (发生错误的行号)
    pub line: usize,
    /// Error message (错误消息)
    pub message: String,
}

impl std::fmt::Display for AssemblyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Line {}: {}", self.line, self.message)
    }
}

/// Symbol information for labels and addresses.
///
/// ---
///
/// 标签和地址的符号信息。
#[derive(Debug, Clone)]
pub struct Symbol {
    /// Symbol name (符号名称)
    pub name: String,
    /// Symbol address (符号地址)
    pub address: u64,
    /// Section where symbol is defined (符号定义的段)
    pub section: Section,
}

/// Memory section types.
///
/// ---
///
/// 内存段类型。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Section {
    /// Code section (.text) (代码段)
    Text,
    /// Initialized data section (.data) (已初始化数据段)
    Data,
    /// Uninitialized data section (.bss) (未初始化数据段)
    Bss,
}

#[derive(Debug, Clone)]
struct PendingRelocation {
    address: u64,
    symbol: String,
    relocation_type: RelocationType,
}

#[derive(Debug, Clone, Copy)]
enum RelocationType {
    Absolute,
    PcRelative,
    Branch,
    Upper,
}

#[derive(Debug, Clone)]
struct AssembledInstruction {
    address: u64,
    machine_code: u32,
    source_line: String,
    line_number: usize,
}

/// RISC-V Assembler
///
/// Assembles RISC-V assembly source into machine code.
/// Performs two-pass assembly: first pass collects labels,
/// second pass encodes instructions.
///
/// ---
///
/// RISC-V 汇编器
///
/// 将 RISC-V 汇编源码汇编为机器码。
/// 执行两遍汇编：第一遍收集标签，第二遍编码指令。
pub struct Assembler {
    symbols: HashMap<String, Symbol>,
    pending_relocations: Vec<PendingRelocation>,
    instructions: Vec<AssembledInstruction>,
    data_section: Vec<u8>,
    bss_size: u64,
    current_section: Section,
    current_address: u64,
    text_base: u64,
    data_base: u64,
    bss_base: u64,
    errors: Vec<AssemblyError>,
}

impl Assembler {
    /// Creates a new Assembler with default settings.
    ///
    /// Default base addresses: text=0, data=0x10000, bss=0x20000.
    ///
    /// ---
    ///
    /// 使用默认设置创建新汇编器。
    ///
    /// 默认基址：text=0, data=0x10000, bss=0x20000。
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            pending_relocations: Vec::new(),
            instructions: Vec::new(),
            data_section: Vec::new(),
            bss_size: 0,
            current_section: Section::Text,
            current_address: 0,
            text_base: 0,
            data_base: 0x10000,
            bss_base: 0x20000,
            errors: Vec::new(),
        }
    }

    /// Sets base addresses for code and data sections.
    ///
    /// ---
    ///
    /// 设置代码和数据段的基址。
    pub fn set_base_addresses(&mut self, text_base: u64, data_base: u64, bss_base: u64) {
        self.text_base = text_base;
        self.data_base = data_base;
        self.bss_base = bss_base;
        self.current_address = text_base;
    }

    /// Assembles an assembly file.
    ///
    /// Reads and assembles the specified file.
    ///
    /// ---
    ///
    /// 汇编汇编文件。
    ///
    /// 读取并汇编指定文件。
    pub fn assemble_file(&mut self, filename: &str) -> Result<Vec<u8>, AssemblyError> {
        let file = std::fs::File::open(filename)
            .map_err(|e| AssemblyError {
                line: 0,
                message: format!("Cannot open file: {}", e),
            })?;
        let reader = io::BufReader::new(file);
        let lines: Vec<String> = reader.lines().filter_map(|l| l.ok()).collect();
        self.assemble(&lines)
    }

    /// Assembles multiple lines of assembly code.
    ///
    /// Performs two-pass assembly and returns machine code.
    ///
    /// ---
    ///
    /// 汇编多行汇编代码。
    ///
    /// 执行两遍汇编并返回机器码。
    pub fn assemble(&mut self, lines: &[String]) -> Result<Vec<u8>, AssemblyError> {
        self.first_pass(lines)?;
        self.second_pass()?;
        
        if !self.errors.is_empty() {
            return Err(self.errors[0].clone());
        }

        Ok(self.link_sections())
    }

    fn first_pass(&mut self, lines: &[String]) -> Result<(), AssemblyError> {
        self.current_section = Section::Text;
        self.current_address = self.text_base;

        for (line_num, line) in lines.iter().enumerate() {
            let line_num = line_num + 1;
            let trimmed = Self::remove_comments(line);
            let trimmed = trimmed.trim();

            if trimmed.is_empty() {
                continue;
            }

            if let Some(directive) = Self::parse_directive(trimmed) {
                self.handle_directive(directive, line_num)?;
                continue;
            }

            if trimmed.ends_with(':') {
                let label = trimmed.trim_end_matches(':').trim().to_string();
                self.symbols.insert(label.clone(), Symbol {
                    name: label,
                    address: self.current_address,
                    section: self.current_section,
                });
                continue;
            }

            if let Some(expanded) = self.expand_pseudo_instruction(trimmed) {
                for exp_line in expanded {
                    self.process_instruction_line(&exp_line, line_num)?;
                }
            } else {
                self.process_instruction_line(trimmed, line_num)?;
            }
        }

        Ok(())
    }

    fn remove_comments(line: &str) -> String {
        if let Some(pos) = line.find('#') {
            line[..pos].to_string()
        } else if let Some(pos) = line.find("//") {
            line[..pos].to_string()
        } else {
            line.to_string()
        }
    }

    fn parse_directive(line: &str) -> Option<&str> {
        let line = line.trim();
        if line.starts_with('.') {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if !parts.is_empty() {
                return Some(parts[0]);
            }
        }
        None
    }

    fn handle_directive(&mut self, directive: &str, line_num: usize) -> Result<(), AssemblyError> {
        match directive {
            ".text" => {
                self.current_section = Section::Text;
                self.current_address = if self.instructions.is_empty() {
                    self.text_base
                } else {
                    self.instructions.last().unwrap().address + 4
                };
            }
            ".data" => {
                self.current_section = Section::Data;
                self.current_address = self.data_base + self.data_section.len() as u64;
            }
            ".bss" => {
                self.current_section = Section::Bss;
                self.current_address = self.bss_base + self.bss_size;
            }
            ".globl" | ".global" => {}
            ".section" => {}
            ".align" => {
                let aligned = (self.current_address + 3) & !3;
                if self.current_section == Section::Bss {
                    self.bss_size += aligned - self.current_address;
                }
                self.current_address = aligned;
            }
            ".byte" => self.handle_data_directive(1, line_num)?,
            ".half" | ".2byte" => self.handle_data_directive(2, line_num)?,
            ".word" | ".4byte" => self.handle_data_directive(4, line_num)?,
            ".dword" | ".8byte" => self.handle_data_directive(8, line_num)?,
            ".quad" => self.handle_data_directive(16, line_num)?,
            ".ascii" => self.handle_ascii_directive(line_num, false)?,
            ".asciz" | ".string" => self.handle_ascii_directive(line_num, true)?,
            ".skip" | ".space" => self.handle_skip_directive(line_num)?,
            ".zero" => self.handle_zero_directive(line_num)?,
            _ => {}
        }
        Ok(())
    }

    fn handle_data_directive(&mut self, size: usize, _line_num: usize) -> Result<(), AssemblyError> {
        self.current_address += size as u64;
        if self.current_section == Section::Bss {
            self.bss_size += size as u64;
        }
        Ok(())
    }

    fn handle_ascii_directive(&mut self, _line_num: usize, null_terminate: bool) -> Result<(), AssemblyError> {
        let extra = if null_terminate { 1 } else { 0 };
        self.current_address += extra as u64;
        if self.current_section == Section::Bss {
            self.bss_size += extra as u64;
        }
        Ok(())
    }

    fn handle_skip_directive(&mut self, _line_num: usize) -> Result<(), AssemblyError> {
        Ok(())
    }

    fn handle_zero_directive(&mut self, _line_num: usize) -> Result<(), AssemblyError> {
        Ok(())
    }

    fn expand_pseudo_instruction(&mut self, line: &str) -> Option<Vec<String>> {
        let parts = Self::tokenize(line);
        if parts.is_empty() {
            return None;
        }

        match parts[0].to_lowercase().as_str() {
            "nop" => Some(vec!["addi x0, x0, 0".to_string()]),
            "mv" => {
                if parts.len() >= 3 {
                    Some(vec![format!("addi {}, {}, 0", parts[1], parts[2])])
                } else { None }
            }
            "not" => {
                if parts.len() >= 3 {
                    Some(vec![format!("xori {}, {}, -1", parts[1], parts[2])])
                } else { None }
            }
            "neg" => {
                if parts.len() >= 3 {
                    Some(vec![format!("sub {}, x0, {}", parts[1], parts[2])])
                } else { None }
            }
            "seqz" => {
                if parts.len() >= 3 {
                    Some(vec![format!("sltiu {}, {}, 1", parts[1], parts[2])])
                } else { None }
            }
            "snez" => {
                if parts.len() >= 3 {
                    Some(vec![format!("sltu {}, x0, {}", parts[1], parts[2])])
                } else { None }
            }
            "sltz" => {
                if parts.len() >= 3 {
                    Some(vec![format!("slt {}, {}, x0", parts[1], parts[2])])
                } else { None }
            }
            "sgtz" => {
                if parts.len() >= 3 {
                    Some(vec![format!("slt {}, x0, {}", parts[1], parts[2])])
                } else { None }
            }
            "beqz" => {
                if parts.len() >= 3 {
                    Some(vec![format!("beq {}, x0, {}", parts[1], parts[2])])
                } else { None }
            }
            "bnez" => {
                if parts.len() >= 3 {
                    Some(vec![format!("bne {}, x0, {}", parts[1], parts[2])])
                } else { None }
            }
            "blez" => {
                if parts.len() >= 3 {
                    Some(vec![format!("ble {}, x0, {}", parts[1], parts[2])])
                } else { None }
            }
            "bgez" => {
                if parts.len() >= 3 {
                    Some(vec![format!("bge {}, x0, {}", parts[1], parts[2])])
                } else { None }
            }
            "bltz" => {
                if parts.len() >= 3 {
                    Some(vec![format!("blt {}, x0, {}", parts[1], parts[2])])
                } else { None }
            }
            "bgtz" => {
                if parts.len() >= 3 {
                    Some(vec![format!("blt x0, {}, {}", parts[1], parts[2])])
                } else { None }
            }
            "j" => {
                if parts.len() >= 2 {
                    Some(vec![format!("jal x0, {}", parts[1])])
                } else { None }
            }
            "jal" if parts.len() == 2 => {
                Some(vec![format!("jal x1, {}", parts[1])])
            }
            "jr" => {
                if parts.len() >= 2 {
                    Some(vec![format!("jalr x0, {}, 0", parts[1])])
                } else { None }
            }
            "jalr" if parts.len() == 2 => {
                Some(vec![format!("jalr x1, {}, 0", parts[1])])
            }
            "ret" => Some(vec!["jalr x0, x1, 0".to_string()]),
            "call" => {
                if parts.len() >= 2 {
                    Some(vec![
                        format!("auipc x6, %pcrel_hi({})", parts[1]),
                        format!("jalr x1, x6, %pcrel_lo({})", parts[1]),
                    ])
                } else { None }
            }
            "tail" => {
                if parts.len() >= 2 {
                    Some(vec![
                        format!("auipc x6, %pcrel_hi({})", parts[1]),
                        format!("jalr x0, x6, %pcrel_lo({})", parts[1]),
                    ])
                } else { None }
            }
            "li" => {
                if parts.len() >= 3 {
                    let imm = self.parse_immediate(&parts[2]).unwrap_or(0);
                    Some(self.expand_li(&parts[1], imm))
                } else { None }
            }
            "la" => {
                if parts.len() >= 3 {
                    Some(vec![
                        format!("auipc {}, %pcrel_hi({})", parts[1], parts[2]),
                        format!("addi {}, {}, %pcrel_lo({})", parts[1], parts[1], parts[2]),
                    ])
                } else { None }
            }
            _ => None,
        }
    }

    fn expand_li(&self, rd: &str, imm: i128) -> Vec<String> {
        if imm == 0 {
            return vec![format!("addi {}, x0, 0", rd)];
        }

        let abs_imm = imm.abs() as u128;
        
        if abs_imm <= 2047 {
            return vec![format!("addi {}, x0, {}", rd, imm)];
        }

        let upper = (imm as u32) & 0xfffff000;
        let lower = (imm as u32) & 0xfff;
        
        if upper == 0 {
            return vec![format!("addi {}, x0, {}", rd, lower)];
        }

        if lower == 0 {
            return vec![format!("lui {}, {}", rd, upper >> 12)];
        }

        vec![
            format!("lui {}, {}", rd, upper >> 12),
            format!("addi {}, {}, {}", rd, rd, Self::sign_extend_12(lower as u16)),
        ]
    }

    fn sign_extend_12(value: u16) -> i16 {
        if value & 0x800 != 0 {
            (value | 0xf000) as i16
        } else {
            value as i16
        }
    }

    fn process_instruction_line(&mut self, line: &str, line_num: usize) -> Result<(), AssemblyError> {
        let parts = Self::tokenize(line);
        if parts.is_empty() {
            return Ok(());
        }

        if self.current_section != Section::Text {
            return Ok(());
        }

        self.instructions.push(AssembledInstruction {
            address: self.current_address,
            machine_code: 0,
            source_line: line.to_string(),
            line_number: line_num,
        });

        self.current_address += 4;
        Ok(())
    }

    fn second_pass(&mut self) -> Result<(), AssemblyError> {
        let encoded_results: Vec<(usize, Result<u32, String>)> = self.instructions
            .iter()
            .enumerate()
            .map(|(idx, instr)| {
                let result = self.encode_instruction(&instr.source_line, instr.address);
                (idx, result)
            })
            .collect();
        
        for (idx, result) in encoded_results {
            match result {
                Ok(code) => self.instructions[idx].machine_code = code,
                Err(e) => self.errors.push(AssemblyError {
                    line: self.instructions[idx].line_number,
                    message: e,
                }),
            }
        }
        Ok(())
    }

    fn encode_instruction(&self, line: &str, address: u64) -> Result<u32, String> {
        let parts = Self::tokenize(line);
        if parts.is_empty() {
            return Err("Empty instruction".to_string());
        }

        let mnemonic = parts[0].to_lowercase();
        
        match mnemonic.as_str() {
            "lui" => self.encode_u_type(&parts, 0x37),
            "auipc" => self.encode_u_type(&parts, 0x17),
            "jal" => self.encode_j_type(&parts, address),
            "jalr" => self.encode_i_type(&parts, 0x67),
            
            "beq" => self.encode_branch(&parts, address, 0x0),
            "bne" => self.encode_branch(&parts, address, 0x1),
            "blt" => self.encode_branch(&parts, address, 0x4),
            "bge" => self.encode_branch(&parts, address, 0x5),
            "bltu" => self.encode_branch(&parts, address, 0x6),
            "bgeu" => self.encode_branch(&parts, address, 0x7),
            
            "lb" => self.encode_load(&parts, 0x0),
            "lh" => self.encode_load(&parts, 0x1),
            "lw" => self.encode_load(&parts, 0x2),
            "ld" => self.encode_load(&parts, 0x3),
            "lq" => self.encode_load(&parts, 0x4),
            "lbu" => self.encode_load(&parts, 0x5),
            "lhu" => self.encode_load(&parts, 0x6),
            "lwu" => self.encode_load(&parts, 0x7),
            "ldu" => self.encode_load(&parts, 0x8),
            
            "sb" => self.encode_store(&parts, 0x0),
            "sh" => self.encode_store(&parts, 0x1),
            "sw" => self.encode_store(&parts, 0x2),
            "sd" => self.encode_store(&parts, 0x3),
            "sq" => self.encode_store(&parts, 0x4),
            
            "addi" => self.encode_imm(&parts, 0x0, false),
            "slli" => self.encode_shift_imm(&parts, 0x1, false),
            "slti" => self.encode_imm(&parts, 0x2, false),
            "sltiu" => self.encode_imm(&parts, 0x3, false),
            "xori" => self.encode_imm(&parts, 0x4, false),
            "srli" => self.encode_shift_imm(&parts, 0x5, false),
            "srai" => self.encode_shift_imm(&parts, 0x5, true),
            "ori" => self.encode_imm(&parts, 0x6, false),
            "andi" => self.encode_imm(&parts, 0x7, false),
            
            "add" => self.encode_r_type(&parts, 0x0, 0x00),
            "sub" => self.encode_r_type(&parts, 0x0, 0x20),
            "sll" => self.encode_r_type(&parts, 0x1, 0x00),
            "slt" => self.encode_r_type(&parts, 0x2, 0x00),
            "sltu" => self.encode_r_type(&parts, 0x3, 0x00),
            "xor" => self.encode_r_type(&parts, 0x4, 0x00),
            "srl" => self.encode_r_type(&parts, 0x5, 0x00),
            "sra" => self.encode_r_type(&parts, 0x5, 0x20),
            "or" => self.encode_r_type(&parts, 0x6, 0x00),
            "and" => self.encode_r_type(&parts, 0x7, 0x00),
            
            "ecall" => Ok(0x00000073),
            "ebreak" => Ok(0x00100073),
            
            // A extension: Atomic operations
            "lr.d" => self.encode_lr(&parts, false),
            "lr.d.aq" => self.encode_lr(&parts, true),
            "sc.d" => self.encode_sc(&parts, false),
            "sc.d.aq" => self.encode_sc(&parts, true),
            "amoadd.d" => self.encode_amo(&parts, 0x00, false, false),
            "amoadd.d.aq" => self.encode_amo(&parts, 0x00, true, false),
            "amoadd.d.rl" => self.encode_amo(&parts, 0x00, false, true),
            "amoadd.d.aqrl" => self.encode_amo(&parts, 0x00, true, true),
            "amoswap.d" => self.encode_amo(&parts, 0x01, false, false),
            "amoswap.d.aq" => self.encode_amo(&parts, 0x01, true, false),
            "amoswap.d.rl" => self.encode_amo(&parts, 0x01, false, true),
            "amoswap.d.aqrl" => self.encode_amo(&parts, 0x01, true, true),
            "amoand.d" => self.encode_amo(&parts, 0x0c, false, false),
            "amoand.d.aq" => self.encode_amo(&parts, 0x0c, true, false),
            "amoand.d.rl" => self.encode_amo(&parts, 0x0c, false, true),
            "amoand.d.aqrl" => self.encode_amo(&parts, 0x0c, true, true),
            "amoor.d" => self.encode_amo(&parts, 0x0a, false, false),
            "amoor.d.aq" => self.encode_amo(&parts, 0x0a, true, false),
            "amoor.d.rl" => self.encode_amo(&parts, 0x0a, false, true),
            "amoor.d.aqrl" => self.encode_amo(&parts, 0x0a, true, true),
            "amoxor.d" => self.encode_amo(&parts, 0x04, false, false),
            "amoxor.d.aq" => self.encode_amo(&parts, 0x04, true, false),
            "amoxor.d.rl" => self.encode_amo(&parts, 0x04, false, true),
            "amoxor.d.aqrl" => self.encode_amo(&parts, 0x04, true, true),
            "amomax.d" => self.encode_amo(&parts, 0x18, false, false),
            "amomax.d.aq" => self.encode_amo(&parts, 0x18, true, false),
            "amomax.d.rl" => self.encode_amo(&parts, 0x18, false, true),
            "amomax.d.aqrl" => self.encode_amo(&parts, 0x18, true, true),
            "amomaxu.d" => self.encode_amo(&parts, 0x1c, false, false),
            "amomaxu.d.aq" => self.encode_amo(&parts, 0x1c, true, false),
            "amomaxu.d.rl" => self.encode_amo(&parts, 0x1c, false, true),
            "amomaxu.d.aqrl" => self.encode_amo(&parts, 0x1c, true, true),
            "amomin.d" => self.encode_amo(&parts, 0x10, false, false),
            "amomin.d.aq" => self.encode_amo(&parts, 0x10, true, false),
            "amomin.d.rl" => self.encode_amo(&parts, 0x10, false, true),
            "amomin.d.aqrl" => self.encode_amo(&parts, 0x10, true, true),
            "amominu.d" => self.encode_amo(&parts, 0x14, false, false),
            "amominu.d.aq" => self.encode_amo(&parts, 0x14, true, false),
            "amominu.d.rl" => self.encode_amo(&parts, 0x14, false, true),
            "amominu.d.aqrl" => self.encode_amo(&parts, 0x14, true, true),
            
            // F extension: Floating-point load/store (single precision)
            "flw" => self.encode_fp_load(&parts, 0x2),
            "fsw" => self.encode_fp_store(&parts, 0x2),
            // D extension: Floating-point load/store (double precision)
            "fld" => self.encode_fp_load(&parts, 0x3),
            "fsd" => self.encode_fp_store(&parts, 0x3),
            // Q extension: Floating-point load/store (quad precision)
            "flq" => self.encode_fp_load(&parts, 0x4),
            "fsq" => self.encode_fp_store(&parts, 0x4),
            
            // F extension: Floating-point arithmetic (single precision)
            "fadd.s" => self.encode_fp_r_type(&parts, 0x00, 0),
            "fsub.s" => self.encode_fp_r_type(&parts, 0x04, 0),
            "fmul.s" => self.encode_fp_r_type(&parts, 0x08, 0),
            "fdiv.s" => self.encode_fp_r_type(&parts, 0x0c, 0),
            "fsqrt.s" => self.encode_fp_sqrt(&parts, 0),
            "fsgnj.s" => self.encode_fp_r_type_funct3(&parts, 0x10, 0, 0x0),
            "fsgnjn.s" => self.encode_fp_r_type_funct3(&parts, 0x10, 0, 0x1),
            "fsgnjx.s" => self.encode_fp_r_type_funct3(&parts, 0x10, 0, 0x2),
            "fmin.s" => self.encode_fp_r_type_funct3(&parts, 0x05, 0, 0x0),
            "fmax.s" => self.encode_fp_r_type_funct3(&parts, 0x05, 0, 0x1),
            "fcvt.w.s" => self.encode_fp_cvt_int(&parts, 0x00, 0),
            "fcvt.wu.s" => self.encode_fp_cvt_int(&parts, 0x04, 0),
            "fcvt.l.s" => self.encode_fp_cvt_int(&parts, 0x08, 0),
            "fcvt.lu.s" => self.encode_fp_cvt_int(&parts, 0x0c, 0),
            "fcvt.s.w" => self.encode_fp_cvt_int(&parts, 0x10, 0),
            "fcvt.s.wu" => self.encode_fp_cvt_int(&parts, 0x14, 0),
            "fcvt.s.l" => self.encode_fp_cvt_int(&parts, 0x18, 0),
            "fcvt.s.lu" => self.encode_fp_cvt_int(&parts, 0x1c, 0),
            "fmv.x.s" => self.encode_fp_mvx(&parts, 0),
            "fmv.s.x" => self.encode_fp_mv_x(&parts, 0),
            "feq.s" => self.encode_fp_cmp(&parts, 0, 0x2),
            "flt.s" => self.encode_fp_cmp(&parts, 0, 0x1),
            "fle.s" => self.encode_fp_cmp(&parts, 0, 0x0),
            "fclass.s" => self.encode_fp_class(&parts, 0),
            
            // D extension: Floating-point arithmetic (double precision)
            "fadd.d" => self.encode_fp_r_type(&parts, 0x00, 1),
            "fsub.d" => self.encode_fp_r_type(&parts, 0x04, 1),
            "fmul.d" => self.encode_fp_r_type(&parts, 0x08, 1),
            "fdiv.d" => self.encode_fp_r_type(&parts, 0x0c, 1),
            "fsqrt.d" => self.encode_fp_sqrt(&parts, 1),
            "fsgnj.d" => self.encode_fp_r_type_funct3(&parts, 0x10, 1, 0x0),
            "fsgnjn.d" => self.encode_fp_r_type_funct3(&parts, 0x10, 1, 0x1),
            "fsgnjx.d" => self.encode_fp_r_type_funct3(&parts, 0x10, 1, 0x2),
            "fmin.d" => self.encode_fp_r_type_funct3(&parts, 0x05, 1, 0x0),
            "fmax.d" => self.encode_fp_r_type_funct3(&parts, 0x05, 1, 0x1),
            "fcvt.w.d" => self.encode_fp_cvt_int(&parts, 0x01, 1),
            "fcvt.wu.d" => self.encode_fp_cvt_int(&parts, 0x05, 1),
            "fcvt.l.d" => self.encode_fp_cvt_int(&parts, 0x09, 1),
            "fcvt.lu.d" => self.encode_fp_cvt_int(&parts, 0x0d, 1),
            "fcvt.d.w" => self.encode_fp_cvt_int(&parts, 0x11, 1),
            "fcvt.d.wu" => self.encode_fp_cvt_int(&parts, 0x15, 1),
            "fcvt.d.l" => self.encode_fp_cvt_int(&parts, 0x19, 1),
            "fcvt.d.lu" => self.encode_fp_cvt_int(&parts, 0x1d, 1),
            "fmv.x.d" => self.encode_fp_mvx(&parts, 1),
            "fmv.d.x" => self.encode_fp_mv_x(&parts, 1),
            "feq.d" => self.encode_fp_cmp(&parts, 1, 0x2),
            "flt.d" => self.encode_fp_cmp(&parts, 1, 0x1),
            "fle.d" => self.encode_fp_cmp(&parts, 1, 0x0),
            "fclass.d" => self.encode_fp_class(&parts, 1),
            "fcvt.d.s" => self.encode_fp_cvt_fp(&parts, 0, 1),  // S -> D
            "fcvt.s.d" => self.encode_fp_cvt_fp(&parts, 1, 0),  // D -> S
            
            // Q extension: Floating-point arithmetic (quad precision)
            "fadd.q" => self.encode_fp_r_type(&parts, 0x00, 3),
            "fsub.q" => self.encode_fp_r_type(&parts, 0x04, 3),
            "fmul.q" => self.encode_fp_r_type(&parts, 0x08, 3),
            "fdiv.q" => self.encode_fp_r_type(&parts, 0x0c, 3),
            "fsqrt.q" => self.encode_fp_sqrt(&parts, 3),
            "fsgnj.q" => self.encode_fp_r_type_funct3(&parts, 0x10, 3, 0x0),
            "fsgnjn.q" => self.encode_fp_r_type_funct3(&parts, 0x10, 3, 0x1),
            "fsgnjx.q" => self.encode_fp_r_type_funct3(&parts, 0x10, 3, 0x2),
            "fmin.q" => self.encode_fp_r_type_funct3(&parts, 0x05, 3, 0x0),
            "fmax.q" => self.encode_fp_r_type_funct3(&parts, 0x05, 3, 0x1),
            "fcvt.w.q" => self.encode_fp_cvt_int(&parts, 0x03, 3),
            "fcvt.wu.q" => self.encode_fp_cvt_int(&parts, 0x07, 3),
            "fcvt.l.q" => self.encode_fp_cvt_int(&parts, 0x0b, 3),
            "fcvt.lu.q" => self.encode_fp_cvt_int(&parts, 0x0f, 3),
            "fcvt.q.w" => self.encode_fp_cvt_int(&parts, 0x13, 3),
            "fcvt.q.wu" => self.encode_fp_cvt_int(&parts, 0x17, 3),
            "fcvt.q.l" => self.encode_fp_cvt_int(&parts, 0x1b, 3),
            "fcvt.q.lu" => self.encode_fp_cvt_int(&parts, 0x1f, 3),
            "fmv.x.q" => self.encode_fp_mvx(&parts, 3),
            "fmv.q.x" => self.encode_fp_mv_x(&parts, 3),
            "feq.q" => self.encode_fp_cmp(&parts, 3, 0x2),
            "flt.q" => self.encode_fp_cmp(&parts, 3, 0x1),
            "fle.q" => self.encode_fp_cmp(&parts, 3, 0x0),
            "fclass.q" => self.encode_fp_class(&parts, 3),
            "fcvt.q.s" => self.encode_fp_cvt_fp(&parts, 0, 3),  // S -> Q
            "fcvt.s.q" => self.encode_fp_cvt_fp(&parts, 3, 0),  // Q -> S
            "fcvt.q.d" => self.encode_fp_cvt_fp(&parts, 1, 3),  // D -> Q
            "fcvt.d.q" => self.encode_fp_cvt_fp(&parts, 3, 1),  // Q -> D
            
            _ => Err(format!("Unknown instruction: {}", mnemonic)),
        }
    }

    fn tokenize(line: &str) -> Vec<String> {
        let line = Self::remove_comments(line);
        let line = line.replace(',', " ");
        
        line.split_whitespace()
            .map(|s| {
                if s.starts_with('%') {
                    s.to_string()
                } else {
                    s.to_string()
                }
            })
            .collect()
    }

    fn parse_register(reg: &str) -> Result<u8, String> {
        let reg = reg.trim_start_matches('x');
        
        match reg.to_lowercase().as_str() {
            "zero" => Ok(0),
            "ra" => Ok(1),
            "sp" => Ok(2),
            "gp" => Ok(3),
            "tp" => Ok(4),
            "t0" => Ok(5),
            "t1" => Ok(6),
            "t2" => Ok(7),
            "s0" | "fp" => Ok(8),
            "s1" => Ok(9),
            "a0" => Ok(10),
            "a1" => Ok(11),
            "a2" => Ok(12),
            "a3" => Ok(13),
            "a4" => Ok(14),
            "a5" => Ok(15),
            "a6" => Ok(16),
            "a7" => Ok(17),
            "s2" => Ok(18),
            "s3" => Ok(19),
            "s4" => Ok(20),
            "s5" => Ok(21),
            "s6" => Ok(22),
            "s7" => Ok(23),
            "s8" => Ok(24),
            "s9" => Ok(25),
            "s10" => Ok(26),
            "s11" => Ok(27),
            "t3" => Ok(28),
            "t4" => Ok(29),
            "t5" => Ok(30),
            "t6" => Ok(31),
            _ => reg.parse::<u8>().map_err(|_| format!("Invalid register: {}", reg)),
        }
    }

    fn parse_immediate(&self, imm_str: &str) -> Result<i128, String> {
        if imm_str.starts_with('%') {
            return Ok(0);
        }
        
        let imm_str = imm_str.trim_start_matches('#');
        
        if let Some(sym) = self.symbols.get(imm_str) {
            return Ok(sym.address as i128);
        }
        
        let negative = imm_str.starts_with('-');
        let imm_str = if negative { &imm_str[1..] } else { imm_str };
        
        let (value, _radix) = if imm_str.starts_with("0x") || imm_str.starts_with("0X") {
            (i128::from_str_radix(&imm_str[2..], 16), 16)
        } else if imm_str.starts_with("0b") || imm_str.starts_with("0B") {
            (i128::from_str_radix(&imm_str[2..], 2), 2)
        } else if imm_str.starts_with("0o") || imm_str.starts_with("0O") {
            (i128::from_str_radix(&imm_str[2..], 8), 8)
        } else {
            (imm_str.parse::<i128>(), 10)
        };
        
        match value {
            Ok(v) => Ok(if negative { -v } else { v }),
            Err(_) => Err(format!("Invalid immediate: {}", imm_str)),
        }
    }

    fn encode_u_type(&self, parts: &[String], opcode: u8) -> Result<u32, String> {
        if parts.len() < 3 {
            return Err("U-type instruction requires rd and imm".to_string());
        }
        
        let rd = Self::parse_register(&parts[1])?;
        let imm = self.parse_immediate(&parts[2])?;
        
        if imm < -(1 << 31) || imm >= (1 << 31) {
            return Err("Immediate out of range".to_string());
        }
        
        let imm_bits = ((imm as u32) >> 12) & 0xfffff;
        Ok((imm_bits << 12) | ((rd as u32) << 7) | opcode as u32)
    }

    fn encode_j_type(&self, parts: &[String], address: u64) -> Result<u32, String> {
        if parts.len() < 2 {
            return Err("J-type instruction requires at least rd".to_string());
        }
        
        let (rd, target_str) = if parts.len() == 2 {
            (1, &parts[1])
        } else {
            (Self::parse_register(&parts[1])?, &parts[2])
        };
        
        let target = self.parse_immediate(target_str)?;
        let offset = target - address as i128;
        
        if offset < -(1 << 20) || offset >= (1 << 20) {
            return Err("Jump offset out of range".to_string());
        }
        
        let offset = offset as u32;
        let imm20 = (offset >> 31) & 0x1;
        let imm10_1 = (offset >> 21) & 0x3ff;
        let imm11 = (offset >> 20) & 0x1;
        let imm19_12 = (offset >> 12) & 0xff;
        
        let encoded = (imm20 << 31) | (imm19_12 << 12) | (imm11 << 20) | (imm10_1 << 21) | ((rd as u32) << 7) | 0x6f;
        Ok(encoded)
    }

    fn encode_i_type(&self, parts: &[String], opcode: u8) -> Result<u32, String> {
        if parts.len() < 3 {
            return Err("I-type instruction requires rd, rs1, and imm".to_string());
        }
        
        let rd = Self::parse_register(&parts[1])?;
        
        if parts[2].contains('(') {
            let offset_end = parts[2].find('(').ok_or("Invalid memory operand")?;
            let offset_str = &parts[2][..offset_end];
            let rs1_end = parts[2].find(')').ok_or("Invalid memory operand")?;
            let rs1_str = &parts[2][offset_end + 1..rs1_end];
            
            let rs1 = Self::parse_register(rs1_str)?;
            let imm = self.parse_immediate(offset_str)?;
            
            if imm < -2048 || imm > 2047 {
                return Err("Immediate out of range for I-type".to_string());
            }
            
            let imm_bits = (imm as u16) as u32;
            return Ok((imm_bits << 20) | ((rs1 as u32) << 15) | ((rd as u32) << 7) | opcode as u32);
        }
        
        let rs1 = Self::parse_register(&parts[2])?;
        let imm = if parts.len() > 3 {
            self.parse_immediate(&parts[3])?
        } else {
            0
        };
        
        if imm < -2048 || imm > 2047 {
            return Err("Immediate out of range for I-type".to_string());
        }
        
        let imm_bits = (imm as u16) as u32;
        Ok((imm_bits << 20) | ((rs1 as u32) << 15) | ((rd as u32) << 7) | opcode as u32)
    }

    fn encode_branch(&self, parts: &[String], address: u64, funct3: u8) -> Result<u32, String> {
        if parts.len() < 4 {
            return Err("Branch instruction requires rs1, rs2, and label".to_string());
        }
        
        let rs1 = Self::parse_register(&parts[1])?;
        let rs2 = Self::parse_register(&parts[2])?;
        let target = self.parse_immediate(&parts[3])?;
        
        let offset = target - address as i128;
        
        if offset < -4096 || offset >= 4096 {
            return Err("Branch offset out of range".to_string());
        }
        
        let offset = offset as u32;
        let imm12 = (offset >> 12) & 0x1;
        let imm10_5 = (offset >> 5) & 0x3f;
        let imm4_1 = (offset >> 1) & 0xf;
        let imm11 = (offset >> 11) & 0x1;
        
        Ok((imm12 << 31) | (imm10_5 << 25) | ((rs2 as u32) << 20) | ((rs1 as u32) << 15) | ((funct3 as u32) << 12) | (imm4_1 << 8) | (imm11 << 7) | 0x63)
    }

    fn encode_load(&self, parts: &[String], funct3: u8) -> Result<u32, String> {
        if parts.len() < 3 {
            return Err("Load instruction requires rd and memory operand".to_string());
        }
        
        let rd = Self::parse_register(&parts[1])?;
        
        let offset_end = parts[2].find('(').ok_or("Invalid memory operand")?;
        let offset_str = &parts[2][..offset_end];
        let rs1_end = parts[2].find(')').ok_or("Invalid memory operand")?;
        let rs1_str = &parts[2][offset_end + 1..rs1_end];
        
        let rs1 = Self::parse_register(rs1_str)?;
        let imm = self.parse_immediate(offset_str)?;
        
        if imm < -2048 || imm > 2047 {
            return Err("Offset out of range for load".to_string());
        }
        
        let imm_bits = (imm as u16) as u32;
        Ok((imm_bits << 20) | ((rs1 as u32) << 15) | ((funct3 as u32) << 12) | ((rd as u32) << 7) | 0x03)
    }

    fn encode_store(&self, parts: &[String], funct3: u8) -> Result<u32, String> {
        if parts.len() < 3 {
            return Err("Store instruction requires rs2 and memory operand".to_string());
        }
        
        let rs2 = Self::parse_register(&parts[1])?;
        
        let offset_end = parts[2].find('(').ok_or("Invalid memory operand")?;
        let offset_str = &parts[2][..offset_end];
        let rs1_end = parts[2].find(')').ok_or("Invalid memory operand")?;
        let rs1_str = &parts[2][offset_end + 1..rs1_end];
        
        let rs1 = Self::parse_register(rs1_str)?;
        let imm = self.parse_immediate(offset_str)?;
        
        if imm < -2048 || imm > 2047 {
            return Err("Offset out of range for store".to_string());
        }
        
        let imm_val = imm as u16 as u32;
        let imm_4_0 = imm_val & 0x1f;
        let imm_11_5 = (imm_val >> 5) & 0x7f;
        
        Ok((imm_11_5 << 25) | ((rs2 as u32) << 20) | ((rs1 as u32) << 15) | ((funct3 as u32) << 12) | (imm_4_0 << 7) | 0x23)
    }

    fn encode_imm(&self, parts: &[String], funct3: u8, _is_arith: bool) -> Result<u32, String> {
        if parts.len() < 4 {
            return Err("Immediate instruction requires rd, rs1, and imm".to_string());
        }
        
        let rd = Self::parse_register(&parts[1])?;
        let rs1 = Self::parse_register(&parts[2])?;
        let imm = self.parse_immediate(&parts[3])?;
        
        if imm < -2048 || imm > 2047 {
            return Err("Immediate out of range".to_string());
        }
        
        let imm_bits = (imm as u16) as u32;
        Ok((imm_bits << 20) | ((rs1 as u32) << 15) | ((funct3 as u32) << 12) | ((rd as u32) << 7) | 0x13)
    }

    fn encode_shift_imm(&self, parts: &[String], funct3: u8, is_arith: bool) -> Result<u32, String> {
        if parts.len() < 4 {
            return Err("Shift instruction requires rd, rs1, and shamt".to_string());
        }
        
        let rd = Self::parse_register(&parts[1])?;
        let rs1 = Self::parse_register(&parts[2])?;
        let shamt = self.parse_immediate(&parts[3])?;
        
        if shamt < 0 || shamt > 127 {
            return Err("Shift amount out of range".to_string());
        }
        
        let funct7 = if is_arith { 0x20u32 } else { 0x00u32 };
        
        Ok((funct7 << 25) | ((shamt as u32 & 0x7f) << 20) | ((rs1 as u32) << 15) | ((funct3 as u32) << 12) | ((rd as u32) << 7) | 0x13)
    }

    fn encode_r_type(&self, parts: &[String], funct3: u8, funct7: u8) -> Result<u32, String> {
        if parts.len() < 4 {
            return Err("R-type instruction requires rd, rs1, and rs2".to_string());
        }
        
        let rd = Self::parse_register(&parts[1])?;
        let rs1 = Self::parse_register(&parts[2])?;
        let rs2 = Self::parse_register(&parts[3])?;
        
        Ok(((funct7 as u32) << 25) | ((rs2 as u32) << 20) | ((rs1 as u32) << 15) | ((funct3 as u32) << 12) | ((rd as u32) << 7) | 0x33)
    }

    // ========================================
    // A Extension Encoding Functions / A 扩展编码函数
    // ========================================

    /// Encodes LR.D (Load Reserved Doubleword)
    /// Format: lr.d rd, (rs1)
    fn encode_lr(&self, parts: &[String], aq: bool) -> Result<u32, String> {
        if parts.len() < 3 {
            return Err("LR.D requires rd and (rs1)".to_string());
        }
        
        let rd = Self::parse_register(&parts[1])?;
        let rs1_str = parts[2].trim_start_matches('(').trim_end_matches(')');
        let rs1 = Self::parse_register(rs1_str)?;
        
        // funct5 = 0x02 (LR), funct3 = 0x2, rs2 = 0
        // funct7 = (funct5 << 2) | (aq << 1) | rl
        let funct7 = (0x02u32 << 2) | (if aq { 0b10 } else { 0 });
        
        Ok((funct7 << 25) | (0u32 << 20) | ((rs1 as u32) << 15) | (0x2u32 << 12) | ((rd as u32) << 7) | 0x2f)
    }

    /// Encodes SC.D (Store Conditional Doubleword)
    /// Format: sc.d rd, rs2, (rs1)
    fn encode_sc(&self, parts: &[String], aq: bool) -> Result<u32, String> {
        if parts.len() < 3 {
            return Err("SC.D requires rd, rs2, and (rs1)".to_string());
        }
        
        let rd = Self::parse_register(&parts[1])?;
        
        // Parse rs2 and (rs1)
        // Format can be: sc.d rd, rs2, (rs1) or sc.d rd, rs2, rs1
        let rs2 = Self::parse_register(&parts[2])?;
        let rs1 = if parts.len() >= 4 {
            let rs1_str = parts[3].trim_start_matches('(').trim_end_matches(')');
            Self::parse_register(rs1_str)?
        } else {
            return Err("SC.D requires rs1 address".to_string());
        };
        
        // funct5 = 0x03 (SC), funct3 = 0x2
        let funct7 = (0x03u32 << 2) | (if aq { 0b10 } else { 0 });
        
        Ok((funct7 << 25) | ((rs2 as u32) << 20) | ((rs1 as u32) << 15) | (0x2u32 << 12) | ((rd as u32) << 7) | 0x2f)
    }

    /// Encodes AMO (Atomic Memory Operation) instructions
    /// Format: amo*.d rd, rs2, (rs1)
    fn encode_amo(&self, parts: &[String], funct5: u8, aq: bool, rl: bool) -> Result<u32, String> {
        if parts.len() < 4 {
            return Err("AMO instruction requires rd, rs2, and (rs1)".to_string());
        }
        
        let rd = Self::parse_register(&parts[1])?;
        let rs2 = Self::parse_register(&parts[2])?;
        let rs1_str = parts[3].trim_start_matches('(').trim_end_matches(')');
        let rs1 = Self::parse_register(rs1_str)?;
        
        // funct7 = (funct5 << 2) | (aq << 1) | rl
        let funct7 = ((funct5 as u32) << 2) | (if aq { 0b10 } else { 0 }) | (if rl { 1 } else { 0 });
        
        Ok((funct7 << 25) | ((rs2 as u32) << 20) | ((rs1 as u32) << 15) | (0x2u32 << 12) | ((rd as u32) << 7) | 0x2f)
    }

    // ========================================
    // F/D/Q Extension Encoding Functions
    // ========================================

    /// Parses a floating-point register (f0-f31)
    fn parse_fp_register(reg: &str) -> Result<u8, String> {
        let reg = reg.trim();
        if reg.starts_with('f') {
            let num_str = &reg[1..];
            num_str.parse::<u8>().map_err(|_| format!("Invalid FP register: {}", reg))
        } else {
            Err(format!("Invalid FP register: {}", reg))
        }
    }

    /// Encodes floating-point load (FLW, FLD, FLQ)
    /// Format: flw/fld/flq fd, offset(rs1)
    fn encode_fp_load(&self, parts: &[String], funct3: u8) -> Result<u32, String> {
        if parts.len() < 3 {
            return Err("FP load requires fd and memory operand".to_string());
        }
        
        let rd = Self::parse_fp_register(&parts[1])?;
        
        let offset_end = parts[2].find('(').ok_or("Invalid memory operand")?;
        let offset_str = &parts[2][..offset_end];
        let rs1_end = parts[2].find(')').ok_or("Invalid memory operand")?;
        let rs1_str = &parts[2][offset_end + 1..rs1_end];
        
        let rs1 = Self::parse_register(rs1_str)?;
        let imm = self.parse_immediate(offset_str)?;
        
        if imm < -2048 || imm > 2047 {
            return Err("Offset out of range for FP load".to_string());
        }
        
        let imm_bits = (imm as u16) as u32;
        // opcode 0x07 for FP load
        Ok((imm_bits << 20) | ((rs1 as u32) << 15) | ((funct3 as u32) << 12) | ((rd as u32) << 7) | 0x07)
    }

    /// Encodes floating-point store (FSW, FSD, FSQ)
    /// Format: fsw/fsd/fsq fs2, offset(rs1)
    fn encode_fp_store(&self, parts: &[String], funct3: u8) -> Result<u32, String> {
        if parts.len() < 3 {
            return Err("FP store requires fs2 and memory operand".to_string());
        }
        
        let rs2 = Self::parse_fp_register(&parts[1])?;
        
        let offset_end = parts[2].find('(').ok_or("Invalid memory operand")?;
        let offset_str = &parts[2][..offset_end];
        let rs1_end = parts[2].find(')').ok_or("Invalid memory operand")?;
        let rs1_str = &parts[2][offset_end + 1..rs1_end];
        
        let rs1 = Self::parse_register(rs1_str)?;
        let imm = self.parse_immediate(offset_str)?;
        
        if imm < -2048 || imm > 2047 {
            return Err("Offset out of range for FP store".to_string());
        }
        
        let imm_bits = (imm as u16) as u32;
        let imm_11_5 = (imm_bits >> 5) & 0x7f;
        let imm_4_0 = imm_bits & 0x1f;
        // opcode 0x27 for FP store
        Ok((imm_11_5 << 25) | ((rs2 as u32) << 20) | ((rs1 as u32) << 15) | ((funct3 as u32) << 12) | (imm_4_0 << 7) | 0x27)
    }

    /// Encodes floating-point R-type instructions (FADD, FSUB, FMUL, FDIV)
    /// Format: fadd.s fd, fs1, fs2
    fn encode_fp_r_type(&self, parts: &[String], funct5: u8, fmt: u8) -> Result<u32, String> {
        if parts.len() < 4 {
            return Err("FP R-type requires fd, fs1, fs2".to_string());
        }
        
        let rd = Self::parse_fp_register(&parts[1])?;
        let rs1 = Self::parse_fp_register(&parts[2])?;
        let rs2 = Self::parse_fp_register(&parts[3])?;
        
        // funct7 = (funct5 << 2) | fmt
        let funct7 = ((funct5 as u32) << 2) | (fmt as u32);
        
        // opcode 0x53 for FP compute
        Ok((funct7 << 25) | ((rs2 as u32) << 20) | ((rs1 as u32) << 15) | (0x0u32 << 12) | ((rd as u32) << 7) | 0x53)
    }

    /// Encodes floating-point R-type instructions with funct3 (FSGNJ, FMIN/FMAX)
    /// Format: fsgnj.s fd, fs1, fs2
    fn encode_fp_r_type_funct3(&self, parts: &[String], funct5: u8, fmt: u8, funct3: u8) -> Result<u32, String> {
        if parts.len() < 4 {
            return Err("FP R-type requires fd, fs1, fs2".to_string());
        }
        
        let rd = Self::parse_fp_register(&parts[1])?;
        let rs1 = Self::parse_fp_register(&parts[2])?;
        let rs2 = Self::parse_fp_register(&parts[3])?;
        
        // funct7 = (funct5 << 2) | fmt
        let funct7 = ((funct5 as u32) << 2) | (fmt as u32);
        
        Ok((funct7 << 25) | ((rs2 as u32) << 20) | ((rs1 as u32) << 15) | ((funct3 as u32) << 12) | ((rd as u32) << 7) | 0x53)
    }

    /// Encodes FSQRT instruction
    /// Format: fsqrt.s fd, fs1
    fn encode_fp_sqrt(&self, parts: &[String], fmt: u8) -> Result<u32, String> {
        if parts.len() < 3 {
            return Err("FSQRT requires fd, fs1".to_string());
        }
        
        let rd = Self::parse_fp_register(&parts[1])?;
        let rs1 = Self::parse_fp_register(&parts[2])?;
        
        // funct5 = 0x0b for FSQRT
        let funct7 = (0x0bu32 << 2) | (fmt as u32);
        
        Ok((funct7 << 25) | (0u32 << 20) | ((rs1 as u32) << 15) | (0x0u32 << 12) | ((rd as u32) << 7) | 0x53)
    }

    /// Encodes FCVT between floating-point and integer
    /// Format: fcvt.w.s rd, fs1
    fn encode_fp_cvt_int(&self, parts: &[String], rs2_val: u8, fmt: u8) -> Result<u32, String> {
        if parts.len() < 3 {
            return Err("FCVT requires rd/fs1".to_string());
        }
        
        // Check if this is FP->Int or Int->FP conversion
        let is_fp_to_int = rs2_val < 0x10;
        
        let (rd, rs1) = if is_fp_to_int {
            // FP to Int: fcvt.w.s x?, f?
            let rd = Self::parse_register(&parts[1])?;
            let rs1 = Self::parse_fp_register(&parts[2])?;
            (rd, rs1)
        } else {
            // Int to FP: fcvt.s.w f?, x?
            let rd = Self::parse_fp_register(&parts[1])?;
            let rs1 = Self::parse_register(&parts[2])?;
            (rd, rs1)
        };
        
        // funct5 = 0x18 for FCVT to/from int
        let funct7 = (0x18u32 << 2) | (fmt as u32);
        
        Ok((funct7 << 25) | ((rs2_val as u32) << 20) | ((rs1 as u32) << 15) | (0x1u32 << 12) | ((rd as u32) << 7) | 0x53)
    }

    /// Encodes FMV.X.S/D/Q (move from FP register to integer register)
    /// Format: fmv.x.s rd, fs1
    fn encode_fp_mvx(&self, parts: &[String], fmt: u8) -> Result<u32, String> {
        if parts.len() < 3 {
            return Err("FMV.X requires rd, fs1".to_string());
        }
        
        let rd = Self::parse_register(&parts[1])?;
        let rs1 = Self::parse_fp_register(&parts[2])?;
        
        // funct5 = 0x1c, rs2 = 0
        let funct7 = (0x1cu32 << 2) | (fmt as u32);
        
        Ok((funct7 << 25) | (0u32 << 20) | ((rs1 as u32) << 15) | (0x0u32 << 12) | ((rd as u32) << 7) | 0x53)
    }

    /// Encodes FMV.S/D/Q.X (move from integer register to FP register)
    /// Format: fmv.s.x fd, rs1
    fn encode_fp_mv_x(&self, parts: &[String], fmt: u8) -> Result<u32, String> {
        if parts.len() < 3 {
            return Err("FMV.X requires fd, rs1".to_string());
        }
        
        let rd = Self::parse_fp_register(&parts[1])?;
        let rs1 = Self::parse_register(&parts[2])?;
        
        // funct5 = 0x1e
        let funct7 = (0x1eu32 << 2) | (fmt as u32);
        
        Ok((funct7 << 25) | (0u32 << 20) | ((rs1 as u32) << 15) | (0x0u32 << 12) | ((rd as u32) << 7) | 0x53)
    }

    /// Encodes FP compare instructions (FEQ, FLT, FLE)
    /// Format: feq.s rd, fs1, fs2
    fn encode_fp_cmp(&self, parts: &[String], fmt: u8, funct3: u8) -> Result<u32, String> {
        if parts.len() < 4 {
            return Err("FP compare requires rd, fs1, fs2".to_string());
        }
        
        let rd = Self::parse_register(&parts[1])?;
        let rs1 = Self::parse_fp_register(&parts[2])?;
        let rs2 = Self::parse_fp_register(&parts[3])?;
        
        // funct5 = 0x14 for compare
        let funct7 = (0x14u32 << 2) | (fmt as u32);
        
        Ok((funct7 << 25) | ((rs2 as u32) << 20) | ((rs1 as u32) << 15) | ((funct3 as u32) << 12) | ((rd as u32) << 7) | 0x53)
    }

    /// Encodes FCLASS instruction
    /// Format: fclass.s rd, fs1
    fn encode_fp_class(&self, parts: &[String], fmt: u8) -> Result<u32, String> {
        if parts.len() < 3 {
            return Err("FCLASS requires rd, fs1".to_string());
        }
        
        let rd = Self::parse_register(&parts[1])?;
        let rs1 = Self::parse_fp_register(&parts[2])?;
        
        // funct5 = 0x1c, rs2 = 1
        let funct7 = (0x1cu32 << 2) | (fmt as u32);
        
        Ok((funct7 << 25) | (1u32 << 20) | ((rs1 as u32) << 15) | (0x1u32 << 12) | ((rd as u32) << 7) | 0x53)
    }

    /// Encodes FP-to-FP conversion (FCVT.D.S, FCVT.S.D, etc.)
    /// Format: fcvt.d.s fd, fs1
    fn encode_fp_cvt_fp(&self, parts: &[String], src_fmt: u8, dst_fmt: u8) -> Result<u32, String> {
        if parts.len() < 3 {
            return Err("FCVT requires fd, fs1".to_string());
        }
        
        let rd = Self::parse_fp_register(&parts[1])?;
        let rs1 = Self::parse_fp_register(&parts[2])?;
        
        // funct5 = 0x08 for FP-to-FP conversion
        // rs2 encodes source format
        let funct7 = (0x08u32 << 2) | (dst_fmt as u32);
        
        Ok((funct7 << 25) | ((src_fmt as u32) << 20) | ((rs1 as u32) << 15) | (0x0u32 << 12) | ((rd as u32) << 7) | 0x53)
    }

    fn link_sections(&self) -> Vec<u8> {
        let text_size = self.instructions.len() * 4;
        let data_size = self.data_section.len();
        
        let mut output = Vec::with_capacity(text_size + data_size);
        
        for instr in &self.instructions {
            output.extend_from_slice(&instr.machine_code.to_le_bytes());
        }
        
        output.extend_from_slice(&self.data_section);
        
        output
    }

    pub fn get_entry_point(&self) -> u64 {
        self.text_base
    }

    pub fn get_symbol_address(&self, name: &str) -> Option<u64> {
        self.symbols.get(name).map(|s| s.address)
    }

    pub fn list_symbols(&self) -> Vec<&Symbol> {
        self.symbols.values().collect()
    }

    pub fn disassemble(&self, code: u32) -> String {
        let opcode = code & 0x7f;
        let rd = ((code >> 7) & 0x1f) as u8;
        let funct3 = ((code >> 12) & 0x7) as u8;
        let rs1 = ((code >> 15) & 0x1f) as u8;
        let rs2 = ((code >> 20) & 0x1f) as u8;
        let funct7 = ((code >> 25) & 0x7f) as u8;
        
        // 先尝试识别伪指令模式
        if let Some(pseudo) = Self::try_recognize_pseudo(code, rd, funct3, rs1, rs2, funct7) {
            return pseudo;
        }
        
        match opcode {
            0x37 => format!("lui x{}, 0x{:05x}", rd, code >> 12),
            0x17 => format!("auipc x{}, 0x{:05x}", rd, code >> 12),
            0x6f => {
                let offset = Self::extract_j_imm(code);
                format!("jal x{}, {:+}", rd, offset)
            }
            0x67 => {
                let imm = ((code >> 20) as i32) as i64;
                format!("jalr x{}, x{}, {}", rd, rs1, imm)
            }
            0x63 => {
                let offset = Self::extract_b_imm(code) as i32;
                let mnemonic = match funct3 {
                    0x0 => "beq",
                    0x1 => "bne",
                    0x4 => "blt",
                    0x5 => "bge",
                    0x6 => "bltu",
                    0x7 => "bgeu",
                    _ => "b???",
                };
                format!("{} x{}, x{}, {:+}", mnemonic, rs1, rs2, offset)
            }
            0x03 => {
                let imm = ((code >> 20) as i32) as i64;
                let mnemonic = match funct3 {
                    0x0 => "lb",
                    0x1 => "lh",
                    0x2 => "lw",
                    0x3 => "ld",
                    0x4 => "lq",
                    0x5 => "lbu",
                    0x6 => "lhu",
                    0x7 => "lwu",
                    _ => "l???",
                };
                format!("{} x{}, {}(x{})", mnemonic, rd, imm, rs1)
            }
            0x23 => {
                let imm = (((code >> 7) & 0x1f) | (((code >> 25) & 0x7f) << 5)) as i16 as i64;
                let mnemonic = match funct3 {
                    0x0 => "sb",
                    0x1 => "sh",
                    0x2 => "sw",
                    0x3 => "sd",
                    0x4 => "sq",
                    _ => "s???",
                };
                format!("{} x{}, {}(x{})", mnemonic, rs2, imm, rs1)
            }
            0x13 => {
                let imm = ((code >> 20) as i32) as i64;
                let shamt = (code >> 20) & 0x7f;
                
                // Check for Zbb/Zba immediate instructions
                if funct7 == 0x30 {
                    // Zbb: RORI
                    if funct3 == 0x5 {
                        format!("rori x{}, x{}, {}", rd, rs1, shamt)
                    } else {
                        format!("???i x{}, x{}, {}", rd, rs1, imm)
                    }
                } else if funct7 == 0x48 {
                    // Zbb: BEXTI
                    if funct3 == 0x5 {
                        format!("bexti x{}, x{}, {}", rd, rs1, shamt)
                    } else {
                        format!("???i x{}, x{}, {}", rd, rs1, imm)
                    }
                } else if funct7 == 0x28 {
                    // Zbb: BCLRI
                    if funct3 == 0x5 {
                        format!("bclri x{}, x{}, {}", rd, rs1, shamt)
                    } else {
                        format!("???i x{}, x{}, {}", rd, rs1, imm)
                    }
                } else if funct7 == 0x08 {
                    // Zba: SLLI.UW
                    if funct3 == 0x1 {
                        format!("slli.uw x{}, x{}, {}", rd, rs1, shamt)
                    } else {
                        format!("???i x{}, x{}, {}", rd, rs1, imm)
                    }
                } else if funct7 == 0x0c {
                    // Zbb: BSETI
                    if funct3 == 0x1 {
                        format!("bseti x{}, x{}, {}", rd, rs1, shamt)
                    } else {
                        format!("???i x{}, x{}, {}", rd, rs1, imm)
                    }
                } else if funct7 == 0x68 {
                    // Zbb: BINVI
                    if funct3 == 0x1 {
                        format!("binvi x{}, x{}, {}", rd, rs1, shamt)
                    } else {
                        format!("???i x{}, x{}, {}", rd, rs1, imm)
                    }
                } else if funct7 == 0x60 {
                    // Zbb: CLZ, CTZ, CPOP, SEXT.B, SEXT.H
                    match funct3 {
                        0x1 => format!("clz x{}, x{}", rd, rs1),
                        0x2 => format!("ctz x{}, x{}", rd, rs1),
                        0x3 => format!("cpop x{}, x{}", rd, rs1),
                        0x4 => format!("sext.b x{}, x{}", rd, rs1),
                        0x5 => format!("sext.h x{}, x{}", rd, rs1),
                        _ => format!("???i x{}, x{}, {}", rd, rs1, imm),
                    }
                } else {
                    // Standard I-type instructions
                    let mnemonic = match funct3 {
                        0x0 => "addi",
                        0x1 => "slli",
                        0x2 => "slti",
                        0x3 => "sltiu",
                        0x4 => "xori",
                        0x5 => if funct7 & 0x20 != 0 { "srai" } else { "srli" },
                        0x6 => "ori",
                        0x7 => "andi",
                        _ => "???i",
                    };
                    if funct3 == 0x1 || funct3 == 0x5 {
                        format!("{} x{}, x{}, {}", mnemonic, rd, rs1, shamt)
                    } else {
                        format!("{} x{}, x{}, {}", mnemonic, rd, rs1, imm)
                    }
                }
            }
            0x33 => {
                // Check for M extension instructions (funct7 == 0x01)
                if funct7 == 0x01 {
                    let mnemonic = match funct3 {
                        0x0 => "mul",
                        0x1 => "mulh",
                        0x2 => "mulhsu",
                        0x3 => "mulhu",
                        0x4 => "div",
                        0x5 => "divu",
                        0x6 => "rem",
                        0x7 => "remu",
                        _ => "???",
                    };
                    format!("{} x{}, x{}, x{}", mnemonic, rd, rs1, rs2)
                } else if funct7 == 0x05 {
                    // Zbc extension: carry-less multiply
                    let mnemonic = match funct3 {
                        0x1 => "clmul",
                        0x2 => "clmulh",
                        0x3 => "clmulr",
                        _ => "???",
                    };
                    format!("{} x{}, x{}, x{}", mnemonic, rd, rs1, rs2)
                } else if funct7 == 0x04 {
                    // Zbb extension: basic bit manipulation (part 1)
                    let mnemonic = match funct3 {
                        0x1 => "rol",
                        0x5 => "ror",
                        _ => "???",
                    };
                    format!("{} x{}, x{}, x{}", mnemonic, rd, rs1, rs2)
                } else if funct7 == 0x20 {
                    // Zbb extension: ANDN, ORN, XORN
                    let mnemonic = match funct3 {
                        0x4 => "xorn",
                        0x6 => "orn",
                        0x7 => "andn",
                        _ => "???",
                    };
                    format!("{} x{}, x{}, x{}", mnemonic, rd, rs1, rs2)
                } else if funct7 == 0x30 {
                    // Zba extension: SH1ADD, SH2ADD, SH3ADD
                    let mnemonic = match funct3 {
                        0x2 => "sh1add",
                        0x4 => "sh2add",
                        0x6 => "sh3add",
                        _ => "???",
                    };
                    format!("{} x{}, x{}, x{}", mnemonic, rd, rs1, rs2)
                } else if funct7 == 0x10 {
                    // Zba extension: SH1ADD.UW, SH2ADD.UW, SH3ADD.UW
                    let mnemonic = match funct3 {
                        0x0 => "add.uw",
                        0x2 => "sh1add.uw",
                        0x4 => "sh2add.uw",
                        0x6 => "sh3add.uw",
                        _ => "???",
                    };
                    format!("{} x{}, x{}, x{}", mnemonic, rd, rs1, rs2)
                } else if funct7 == 0x08 {
                    // Zba extension: SLLI.UW is I-type, this is for other instructions
                    "???".to_string()
                } else if funct7 == 0x40 {
                    // Zbb extension: MAX, MAXU, MIN, MINU
                    let mnemonic = match funct3 {
                        0x4 => "min",
                        0x5 => "minu",
                        0x6 => "max",
                        0x7 => "maxu",
                        _ => "???",
                    };
                    format!("{} x{}, x{}, x{}", mnemonic, rd, rs1, rs2)
                } else if (funct7 & 0x80) != 0 {
                    // P extension: SIMD instructions have bit 7 set in funct7
                    Self::disassemble_simd(funct7, funct3, rd, rs1, rs2)
                } else {
                    // Standard R-type instructions
                    let mnemonic = match funct3 {
                        0x0 => if funct7 & 0x20 != 0 { "sub" } else { "add" },
                        0x1 => "sll",
                        0x2 => "slt",
                        0x3 => "sltu",
                        0x4 => "xor",
                        0x5 => if funct7 & 0x20 != 0 { "sra" } else { "srl" },
                        0x6 => "or",
                        0x7 => "and",
                        _ => "???",
                    };
                    format!("{} x{}, x{}, x{}", mnemonic, rd, rs1, rs2)
                }
            }
            0x73 => {
                if funct3 == 0 {
                    let imm = (code >> 20) & 0xfff;
                    match imm {
                        0 => "ecall".to_string(),
                        1 => "ebreak".to_string(),
                        _ => format!("system 0x{:03x}", imm),
                    }
                } else {
                    // Zicsr extension: CSR instructions
                    let csr = (code >> 20) & 0xfff;
                    let csr_name = register::CsrAddress::from_u16(csr as u16).name();
                    match funct3 {
                        0x1 => format!("csrrw x{}, x{}, {}", rd, rs1, csr_name),
                        0x2 => format!("csrrs x{}, x{}, {}", rd, rs1, csr_name),
                        0x3 => format!("csrrc x{}, x{}, {}", rd, rs1, csr_name),
                        0x5 => format!("csrrwi x{}, 0x{:x}, {}", rd, rs1, csr_name),
                        0x6 => format!("csrrsi x{}, 0x{:x}, {}", rd, rs1, csr_name),
                        0x7 => format!("csrrci x{}, 0x{:x}, {}", rd, rs1, csr_name),
                        _ => format!("csr ??? 0x{:03x}", csr),
                    }
                }
            }
            // Zifencei extension: Misc-Mem operations (opcode 0x0f)
            0x0f => {
                match funct3 {
                    0x0 => {
                        // FENCE: pred=succ format
                        let pred = (funct7 >> 4) & 0xf;
                        let succ = funct7 & 0xf;
                        format!("fence 0x{:x}, 0x{:x}", pred, succ)
                    }
                    0x1 => "fence.i".to_string(),
                    _ => format!("miscmem ??? funct3={}", funct3),
                }
            }
            // A extension: Atomic operations (opcode 0x2f)
            0x2f => {
                let funct5 = (funct7 >> 2) & 0x1f;
                let aq = (funct7 >> 1) & 1;
                let rl = funct7 & 1;
                let suffix = if aq != 0 && rl != 0 {
                    ".aqrl"
                } else if aq != 0 {
                    ".aq"
                } else if rl != 0 {
                    ".rl"
                } else {
                    ""
                };
                
                match funct5 {
                    0x02 => {
                        // LR.D: Load Reserved
                        // Format: lr.d rd, (rs1)
                        if rs2 == 0 {
                            format!("lr.d{} x{}, (x{})", suffix, rd, rs1)
                        } else {
                            format!("lr.d{} x{}, (x{}) # invalid rs2={}", suffix, rd, rs1, rs2)
                        }
                    }
                    0x03 => {
                        // SC.D: Store Conditional
                        // Format: sc.d rd, rs2, (rs1)
                        format!("sc.d{} x{}, x{}, (x{})", suffix, rd, rs2, rs1)
                    }
                    0x00 => {
                        // AMOADD.D: Atomic Add
                        format!("amoadd.d{} x{}, x{}, (x{})", suffix, rd, rs2, rs1)
                    }
                    0x01 => {
                        // AMOSWAP.D: Atomic Swap
                        format!("amoswap.d{} x{}, x{}, (x{})", suffix, rd, rs2, rs1)
                    }
                    0x0c => {
                        // AMOAND.D: Atomic AND
                        format!("amoand.d{} x{}, x{}, (x{})", suffix, rd, rs2, rs1)
                    }
                    0x0a => {
                        // AMOOR.D: Atomic OR
                        format!("amoor.d{} x{}, x{}, (x{})", suffix, rd, rs2, rs1)
                    }
                    0x04 => {
                        // AMOXOR.D: Atomic XOR
                        format!("amoxor.d{} x{}, x{}, (x{})", suffix, rd, rs2, rs1)
                    }
                    0x18 => {
                        // AMOMAX.D: Atomic Maximum (signed)
                        format!("amomax.d{} x{}, x{}, (x{})", suffix, rd, rs2, rs1)
                    }
                    0x1c => {
                        // AMOMAXU.D: Atomic Maximum (unsigned)
                        format!("amomaxu.d{} x{}, x{}, (x{})", suffix, rd, rs2, rs1)
                    }
                    0x10 => {
                        // AMOMIN.D: Atomic Minimum (signed)
                        format!("amomin.d{} x{}, x{}, (x{})", suffix, rd, rs2, rs1)
                    }
                    0x14 => {
                        // AMOMINU.D: Atomic Minimum (unsigned)
                        format!("amominu.d{} x{}, x{}, (x{})", suffix, rd, rs2, rs1)
                    }
                    _ => format!("amo_unknown funct5=0x{:02x}", funct5),
                }
            }
            // F/D/Q extension: Floating-point load (opcode 0x07)
            0x07 => {
                let imm = ((code >> 20) as i32) as i64;
                let mnemonic = match funct3 {
                    0x2 => "flw",
                    0x3 => "fld",
                    0x4 => "flq",
                    _ => "fl?",
                };
                format!("{} f{}, {}(x{})", mnemonic, rd, imm, rs1)
            }
            // F/D/Q extension: Floating-point store (opcode 0x27)
            0x27 => {
                let imm = (((code >> 7) & 0x1f) | (((code >> 25) & 0x7f) << 5)) as i16 as i64;
                let mnemonic = match funct3 {
                    0x2 => "fsw",
                    0x3 => "fsd",
                    0x4 => "fsq",
                    _ => "fs?",
                };
                format!("{} f{}, {}(x{})", mnemonic, rs2, imm, rs1)
            }
            // F/D/Q extension: Floating-point compute (opcode 0x53)
            0x53 => {
                let funct5 = (funct7 >> 2) & 0x1f;
                let fmt = funct7 & 0x3;
                let suffix = match fmt {
                    0 => ".s",
                    1 => ".d",
                    3 => ".q",
                    _ => ".?",
                };
                
                match funct5 {
                    // FADD
                    0x00 => format!("fadd{} f{}, f{}, f{}", suffix, rd, rs1, rs2),
                    // FSUB
                    0x04 => format!("fsub{} f{}, f{}, f{}", suffix, rd, rs1, rs2),
                    // FMUL
                    0x08 => format!("fmul{} f{}, f{}, f{}", suffix, rd, rs1, rs2),
                    // FDIV
                    0x0c => format!("fdiv{} f{}, f{}, f{}", suffix, rd, rs1, rs2),
                    // FSQRT
                    0x0b => format!("fsqrt{} f{}, f{}", suffix, rd, rs1),
                    // FSGNJ, FSGNJN, FSGNJX (funct5 = 0x10)
                    0x10 => match funct3 {
                        0x0 => format!("fsgnj{} f{}, f{}, f{}", suffix, rd, rs1, rs2),
                        0x1 => format!("fsgnjn{} f{}, f{}, f{}", suffix, rd, rs1, rs2),
                        0x2 => format!("fsgnjx{} f{}, f{}, f{}", suffix, rd, rs1, rs2),
                        _ => format!("fsgnj?{} f{}, f{}, f{}", suffix, rd, rs1, rs2),
                    },
                    // FMIN/FMAX
                    0x05 => match funct3 {
                        0x0 => format!("fmin{} f{}, f{}, f{}", suffix, rd, rs1, rs2),
                        0x1 => format!("fmax{} f{}, f{}, f{}", suffix, rd, rs1, rs2),
                        _ => format!("fminmax?{} f{}, f{}, f{}", suffix, rd, rs1, rs2),
                    },
                    // FEQ/FLT/FLE
                    0x14 => match funct3 {
                        0x0 => format!("fle{} x{}, f{}, f{}", suffix, rd, rs1, rs2),
                        0x1 => format!("flt{} x{}, f{}, f{}", suffix, rd, rs1, rs2),
                        0x2 => format!("feq{} x{}, f{}, f{}", suffix, rd, rs1, rs2),
                        _ => format!("fcmp?{} x{}, f{}, f{}", suffix, rd, rs1, rs2),
                    },
                    // FCVT.X.S/D/Q, FCLASS.S/D/Q, FMV.X.S/D/Q
                    0x18 => {
                        if rs2 == 0 {
                            format!("fmv.x{} x{}, f{}", suffix, rd, rs1)
                        } else if rs2 == 1 {
                            format!("fclass{} x{}, f{}", suffix, rd, rs1)
                        } else {
                            format!("fcvt.??? x{}, f{}, rs2={}", rd, rs1, rs2)
                        }
                    }
                    // FMV.S/D/Q.X
                    0x1e => match funct3 {
                        0x0 => format!("fmv{}.x f{}, x{}", suffix, rd, rs1),
                        _ => format!("fmv?.x f{}, x{}", rd, rs1),
                    },
                    // FCVT.W/L <-> S/D/Q
                    0x1c => Self::disassemble_fcvt_int(rd, rs1, rs2, fmt),
                    _ => format!("fp_unk{} funct5=0x{:02x} f{}, f{}, f{}", suffix, funct5, rd, rs1, rs2),
                }
            }
            // V extension: Vector operations (opcode 0x57)
            0x57 => {
                let funct6 = (funct7 >> 1) & 0x3f;
                Self::disassemble_vector(funct6, funct3, rd, rs1, rs2)
            }
            _ => format!("unknown opcode 0x{:02x}", opcode),
        }
    }

    /// Disassemble vector instructions.
    fn disassemble_vector(funct6: u8, funct3: u8, rd: u8, rs1: u8, rs2: u8) -> String {
        match funct3 {
            // OPIVV: Vector-Vector integer
            0x0 => match funct6 {
                0x00 => format!("vadd.vv v{}, v{}, v{}", rd, rs1, rs2),
                0x02 => format!("vsub.vv v{}, v{}, v{}", rd, rs1, rs2),
                0x04 => format!("vmin.vv v{}, v{}, v{}", rd, rs1, rs2),
                0x05 => format!("vminu.vv v{}, v{}, v{}", rd, rs1, rs2),
                0x06 => format!("vmax.vv v{}, v{}, v{}", rd, rs1, rs2),
                0x07 => format!("vmaxu.vv v{}, v{}, v{}", rd, rs1, rs2),
                0x08 => format!("vand.vv v{}, v{}, v{}", rd, rs1, rs2),
                0x09 => format!("vor.vv v{}, v{}, v{}", rd, rs1, rs2),
                0x0a => format!("vxor.vv v{}, v{}, v{}", rd, rs1, rs2),
                0x0c => format!("vsll.vv v{}, v{}, v{}", rd, rs1, rs2),
                0x0d => format!("vsrl.vv v{}, v{}, v{}", rd, rs1, rs2),
                0x0e => format!("vsra.vv v{}, v{}, v{}", rd, rs1, rs2),
                0x18 => format!("vmv.v.v v{}, v{}", rd, rs2),
                _ => format!("v_opivv v{}, v{}, v{} funct6=0x{:02x}", rd, rs1, rs2, funct6),
            },
            // OPFVV: Vector-Vector floating-point
            0x1 => match funct6 {
                0x00 => format!("vfadd.vv v{}, v{}, v{}", rd, rs1, rs2),
                0x02 => format!("vfsub.vv v{}, v{}, v{}", rd, rs1, rs2),
                0x04 => format!("vfmin.vv v{}, v{}, v{}", rd, rs1, rs2),
                0x05 => format!("vfmax.vv v{}, v{}, v{}", rd, rs1, rs2),
                0x18 => format!("vfmv.v.f v{}, f{}", rd, rs1),
                _ => format!("v_opfvv v{}, v{}, v{} funct6=0x{:02x}", rd, rs1, rs2, funct6),
            },
            // OPMVV: Vector multiply/divide
            0x2 => match funct6 {
                0x0d => format!("vdiv.vv v{}, v{}, v{}", rd, rs1, rs2),
                0x0f => format!("vrem.vv v{}, v{}, v{}", rd, rs1, rs2),
                0x11 => format!("vmul.vv v{}, v{}, v{}", rd, rs1, rs2),
                0x13 => format!("vmulh.vv v{}, v{}, v{}", rd, rs1, rs2),
                0x18 => format!("vmacc.vv v{}, v{}, v{}", rd, rs1, rs2),
                _ => format!("v_opmvv v{}, v{}, v{} funct6=0x{:02x}", rd, rs1, rs2, funct6),
            },
            // OPVI: Vector-Immediate integer
            0x3 => match funct6 {
                0x00 => format!("vadd.vx v{}, v{}, x{}", rd, rs2, rs1),
                0x02 => format!("vsub.vx v{}, v{}, x{}", rd, rs2, rs1),
                0x08 => format!("vand.vx v{}, v{}, x{}", rd, rs2, rs1),
                0x09 => format!("vor.vx v{}, v{}, x{}", rd, rs2, rs1),
                0x0a => format!("vxor.vx v{}, v{}, x{}", rd, rs2, rs1),
                0x0c => format!("vsll.vx v{}, v{}, x{}", rd, rs2, rs1),
                0x0d => format!("vsrl.vx v{}, v{}, x{}", rd, rs2, rs1),
                0x0e => format!("vsra.vx v{}, v{}, x{}", rd, rs2, rs1),
                _ => format!("v_opvi v{}, v{}, x{} funct6=0x{:02x}", rd, rs2, rs1, funct6),
            },
            // OPIVI: Vector-5-bit immediate
            0x4 => {
                let imm = ((rs1 as i8) << 3 >> 3) as i16;
                match funct6 {
                    0x00 => format!("vadd.vi v{}, v{}, {}", rd, rs2, imm),
                    0x08 => format!("vand.vi v{}, v{}, {}", rd, rs2, imm),
                    0x09 => format!("vor.vi v{}, v{}, {}", rd, rs2, imm),
                    0x0a => format!("vxor.vi v{}, v{}, {}", rd, rs2, imm),
                    0x0c => format!("vsll.vi v{}, v{}, {}", rd, rs2, imm),
                    0x0d => format!("vsrl.vi v{}, v{}, {}", rd, rs2, imm),
                    0x0e => format!("vsra.vi v{}, v{}, {}", rd, rs2, imm),
                    _ => format!("v_opivi v{}, v{}, {} funct6=0x{:02x}", rd, rs2, imm, funct6),
                }
            }
            // OPFVF: Vector-Scalar FP
            0x5 => match funct6 {
                0x00 => format!("vfadd.vf v{}, v{}, f{}", rd, rs2, rs1),
                0x02 => format!("vfsub.vf v{}, v{}, f{}", rd, rs2, rs1),
                0x04 => format!("vfmin.vf v{}, v{}, f{}", rd, rs2, rs1),
                0x05 => format!("vfmax.vf v{}, v{}, f{}", rd, rs2, rs1),
                0x18 => format!("vfmv.v.f v{}, f{}", rd, rs1),
                _ => format!("v_opfvf v{}, v{}, f{} funct6=0x{:02x}", rd, rs2, rs1, funct6),
            },
            // Vector load/store
            0x6 => match funct6 {
                0x00 => format!("vle8.v v{}, (x{})", rd, rs1),
                0x01 => format!("vle16.v v{}, (x{})", rd, rs1),
                0x02 => format!("vle32.v v{}, (x{})", rd, rs1),
                0x03 => format!("vle64.v v{}, (x{})", rd, rs1),
                0x04 => format!("vle128.v v{}, (x{})", rd, rs1),
                _ => format!("v_load/store v{}, (x{}) funct6=0x{:02x}", rd, rs1, funct6),
            },
            // Vector configuration
            0x7 => match funct6 {
                0x30 => format!("vsetvli x{}, x{}, 0x{:02x}", rd, rs1, rs2),
                0x31 => format!("vsetivli x{}, {}, 0x{:02x}", rd, rs1, rs2),
                0x3f => format!("vsetvl x{}, x{}, x{}", rd, rs1, rs2),
                _ => format!("v_cfg funct6=0x{:02x}", funct6),
            },
            _ => format!("v_unknown funct3=0x{:02x}", funct3),
        }
    }

    /// Disassemble SIMD instruction (P extension).
    fn disassemble_simd(funct7: u8, funct3: u8, rd: u8, rs1: u8, rs2: u8) -> String {
        let mnemonic = match funct7 {
            // SIMD arithmetic operations
            0x80 => match funct3 {
                0x0 => "add8",
                0x1 => "add16",
                0x2 => "add32",
                0x3 => "add64",
                _ => "simd_add?",
            },
            0x81 => match funct3 {
                0x0 => "sub8",
                0x1 => "sub16",
                0x2 => "sub32",
                0x3 => "sub64",
                _ => "simd_sub?",
            },
            0x82 => match funct3 {
                0x0 => "mul8",
                0x1 => "mul16",
                0x2 => "mul32",
                0x3 => "mul64",
                _ => "simd_mul?",
            },
            0x83 => match funct3 {
                0x0 => "sll8",
                0x1 => "sll16",
                0x2 => "sll32",
                0x3 => "sll64",
                _ => "simd_sll?",
            },
            0x84 => match funct3 {
                0x0 => "srl8",
                0x1 => "srl16",
                0x2 => "srl32",
                0x3 => "srl64",
                _ => "simd_srl?",
            },
            0x85 => match funct3 {
                0x0 => "sra8",
                0x1 => "sra16",
                0x2 => "sra32",
                0x3 => "sra64",
                _ => "simd_sra?",
            },
            0x86 => match funct3 {
                0x0 => "and8",
                0x1 => "and16",
                0x2 => "and32",
                0x3 => "and64",
                _ => "simd_and?",
            },
            0x87 => match funct3 {
                0x0 => "or8",
                0x1 => "or16",
                0x2 => "or32",
                0x3 => "or64",
                _ => "simd_or?",
            },
            0x88 => match funct3 {
                0x0 => "xor8",
                0x1 => "xor16",
                0x2 => "xor32",
                0x3 => "xor64",
                _ => "simd_xor?",
            },
            0x89 => match funct3 {
                0x0 => "cmpeq8",
                0x1 => "cmpeq16",
                0x2 => "cmpeq32",
                0x3 => "cmpeq64",
                _ => "simd_cmpeq?",
            },
            0x8a => match funct3 {
                0x0 => "cmplt8",
                0x1 => "cmplt16",
                0x2 => "cmplt32",
                0x3 => "cmplt64",
                _ => "simd_cmplt?",
            },
            0x8b => match funct3 {
                0x0 => "cmpltu8",
                0x1 => "cmpltu16",
                0x2 => "cmpltu32",
                0x3 => "cmpltu64",
                _ => "simd_cmpltu?",
            },
            0x8c => match funct3 {
                0x0 => "min8",
                0x1 => "min16",
                0x2 => "min32",
                0x3 => "min64",
                _ => "simd_min?",
            },
            0x8d => match funct3 {
                0x0 => "minu8",
                0x1 => "minu16",
                0x2 => "minu32",
                0x3 => "minu64",
                _ => "simd_minu?",
            },
            0x8e => match funct3 {
                0x0 => "max8",
                0x1 => "max16",
                0x2 => "max32",
                0x3 => "max64",
                _ => "simd_max?",
            },
            0x8f => match funct3 {
                0x0 => "maxu8",
                0x1 => "maxu16",
                0x2 => "maxu32",
                0x3 => "maxu64",
                _ => "simd_maxu?",
            },
            _ => "simd?",
        };
        format!("{} x{}, x{}, x{}", mnemonic, rd, rs1, rs2)
    }

    /// Disassemble 16-bit compressed instruction.
    pub fn disassemble_compressed(code: u16) -> String {
        let opcode = (code & 0x3) as u8;
        let funct3 = ((code >> 13) & 0x7) as u8;
        let rd = ((code >> 7) & 0x1f) as u8;
        let rs2 = ((code >> 2) & 0x1f) as u8;
        
        match opcode {
            0x0 => match funct3 {
                0x0 => {
                    let nzuimm = (((code >> 3) & 0x1f) | 
                                 (((code >> 5) & 0x1) << 5) |
                                 (((code >> 6) & 0x1) << 6) |
                                 (((code >> 11) & 0x1) << 7)) as u16;
                    if nzuimm == 0 {
                        "c.illegal".to_string()
                    } else {
                        format!("c.addi4spn x{}, x2, {}", rd + 8, nzuimm)
                    }
                }
                0x2 => {
                    let offset = ((((code >> 6) & 0x7) << 2) |
                                 (((code >> 10) & 0x3) << 6)) as u16;
                    format!("c.lw x{}, {}(x{})", rd + 8, offset, rs2 + 8)
                }
                0x3 => {
                    let offset = ((((code >> 6) & 0x7) << 3) |
                                 (((code >> 10) & 0x7) << 6)) as u16;
                    format!("c.ld x{}, {}(x{})", rd + 8, offset, rs2 + 8)
                }
                0x6 => {
                    let offset = ((((code >> 6) & 0x7) << 2) |
                                 (((code >> 10) & 0x3) << 6)) as u16;
                    format!("c.sw x{}, {}(x{})", rs2 + 8, offset, rd + 8)
                }
                0x7 => {
                    let offset = ((((code >> 6) & 0x7) << 3) |
                                 (((code >> 10) & 0x7) << 6)) as u16;
                    format!("c.sd x{}, {}(x{})", rs2 + 8, offset, rd + 8)
                }
                _ => format!("c.q0 funct3=0x{:02x}", funct3),
            },
            0x1 => match funct3 {
                0x0 => {
                    let imm = ((((code >> 12) & 0x1) << 5) | ((code >> 2) & 0x1f)) as i8 as i16;
                    if rd == 0 {
                        format!("c.nop")
                    } else {
                        format!("c.addi x{}, {}", rd, imm)
                    }
                }
                0x1 => {
                    let offset = (((code >> 2) & 0x7ff) as i16) << 1;
                    format!("c.jal {:+}", offset)
                }
                0x2 => {
                    let imm = ((((code >> 12) & 0x1) << 5) | ((code >> 2) & 0x1f)) as i8 as i16;
                    format!("c.li x{}, {}", rd, imm)
                }
                0x3 => {
                    if rd == 2 {
                        let nzimm = (((((code >> 12) & 0x1) << 9) |
                                     (((code >> 3) & 0x3) << 7) |
                                     (((code >> 5) & 0x1) << 6) |
                                     (((code >> 2) & 0x1) << 5) |
                                     (((code >> 6) & 0x1) << 4)) as i16) << 4;
                        format!("c.addi16sp x2, {}", nzimm)
                    } else {
                        let nzimm = ((((code >> 12) & 0x1) as u32) << 17) | ((((code >> 2) & 0x1f) as u32) << 12);
                        format!("c.lui x{}, 0x{:x}", rd, nzimm >> 12)
                    }
                }
                0x4 => {
                    let funct2 = ((code >> 10) & 0x3) as u8;
                    match funct2 {
                        0x0 | 0x1 => {
                            let shamt = ((((code >> 12) & 0x1) << 5) | ((code >> 2) & 0x1f)) as u8;
                            if funct2 == 0x1 {
                                format!("c.srai x{}, {}", rs2 + 8, shamt)
                            } else {
                                format!("c.srli x{}, {}", rs2 + 8, shamt)
                            }
                        }
                        0x2 => {
                            let imm = ((((code >> 12) & 0x1) << 5) | ((code >> 2) & 0x1f)) as i8 as i16;
                            format!("c.andi x{}, {}", rs2 + 8, imm)
                        }
                        0x3 => {
                            let funct2_sub = ((code >> 5) & 0x3) as u8;
                            match funct2_sub {
                                0x0 => format!("c.sub x{}, x{}", rs2 + 8, rd),
                                0x1 => format!("c.xor x{}, x{}", rs2 + 8, rd),
                                0x2 => format!("c.or x{}, x{}", rs2 + 8, rd),
                                0x3 => format!("c.and x{}, x{}", rs2 + 8, rd),
                                _ => "c.unknown".to_string(),
                            }
                        }
                        _ => "c.q1.unk".to_string(),
                    }
                }
                0x5 => {
                    let offset = (((code >> 2) & 0x7ff) as i16) << 1;
                    format!("c.j {:+}", offset)
                }
                0x6 => {
                    let offset = ((((code >> 2) & 0x1) |
                                 (((code >> 3) & 0x3) << 1) |
                                 (((code >> 5) & 0x3) << 3) |
                                 (((code >> 10) & 0x3) << 5) |
                                 (((code >> 12) & 0x1) << 7)) as i16) << 1;
                    format!("c.beqz x{}, {:+}", rs2 + 8, offset)
                }
                0x7 => {
                    let offset = ((((code >> 2) & 0x1) |
                                 (((code >> 3) & 0x3) << 1) |
                                 (((code >> 5) & 0x3) << 3) |
                                 (((code >> 10) & 0x3) << 5) |
                                 (((code >> 12) & 0x1) << 7)) as i16) << 1;
                    format!("c.bnez x{}, {:+}", rs2 + 8, offset)
                }
                _ => format!("c.q1 funct3=0x{:02x}", funct3),
            },
            0x2 => match funct3 {
                0x0 => {
                    let shamt = ((((code >> 12) & 0x1) << 5) | ((code >> 2) & 0x1f)) as u8;
                    format!("c.slli x{}, {}", rd, shamt)
                }
                0x2 => {
                    let offset = ((((code >> 2) & 0x7) << 2) |
                                 (((code >> 12) & 0x1) << 5) |
                                 (((code >> 5) & 0x3) << 6)) as u16;
                    format!("c.lwsp x{}, {}(x2)", rd, offset)
                }
                0x3 => {
                    let offset = ((((code >> 2) & 0x7) << 3) |
                                 (((code >> 12) & 0x1) << 5) |
                                 (((code >> 5) & 0x7) << 6)) as u16;
                    format!("c.ldsp x{}, {}(x2)", rd, offset)
                }
                0x4 => {
                    if ((code >> 12) & 0x1) == 0 {
                        if rs2 == 0 {
                            format!("c.jr x{}", rd)
                        } else {
                            format!("c.mv x{}, x{}", rd, rs2)
                        }
                    } else {
                        if rd == 0 && rs2 == 0 {
                            "c.ebreak".to_string()
                        } else if rs2 == 0 {
                            format!("c.jalr x{}", rd)
                        } else {
                            format!("c.add x{}, x{}", rd, rs2)
                        }
                    }
                }
                0x6 => {
                    let offset = ((((code >> 2) & 0x7) << 2) |
                                 (((code >> 9) & 0xf) << 5)) as u16;
                    format!("c.swsp x{}, {}(x2)", rs2, offset)
                }
                0x7 => {
                    let offset = ((((code >> 2) & 0x7) << 3) |
                                 (((code >> 10) & 0x7) << 6) |
                                 (((code >> 9) & 0x1) << 9)) as u16;
                    format!("c.sdsp x{}, {}(x2)", rs2, offset)
                }
                _ => format!("c.q2 funct3=0x{:02x}", funct3),
            },
            _ => format!("c.illegal 0x{:04x}", code),
        }
    }

    /// Disassemble FCVT instructions (floating-point to floating-point)
    fn disassemble_fcvt(rd: u8, rs1: u8, rs2: u8, dst_fmt: u8) -> String {
        let dst_suffix = match dst_fmt {
            0 => ".s",
            1 => ".d",
            3 => ".q",
            _ => ".?",
        };
        let src_suffix = match rs2 {
            0 => ".s",
            1 => ".d",
            3 => ".q",
            _ => ".?",
        };
        format!("fcvt{}{} f{}, f{}", dst_suffix, src_suffix, rd, rs1)
    }

    /// Disassemble FCVT instructions (floating-point to/from integer)
    fn disassemble_fcvt_int(rd: u8, rs1: u8, rs2: u8, _fmt: u8) -> String {
        match rs2 {
            // To signed 32-bit
            0x00 => format!("fcvt.w.s x{}, f{}", rd, rs1),
            0x01 => format!("fcvt.w.d x{}, f{}", rd, rs1),
            0x03 => format!("fcvt.w.q x{}, f{}", rd, rs1),
            // To unsigned 32-bit
            0x04 => format!("fcvt.wu.s x{}, f{}", rd, rs1),
            0x05 => format!("fcvt.wu.d x{}, f{}", rd, rs1),
            0x07 => format!("fcvt.wu.q x{}, f{}", rd, rs1),
            // To signed 64-bit
            0x08 => format!("fcvt.l.s x{}, f{}", rd, rs1),
            0x09 => format!("fcvt.l.d x{}, f{}", rd, rs1),
            0x0b => format!("fcvt.l.q x{}, f{}", rd, rs1),
            // To unsigned 64-bit
            0x0c => format!("fcvt.lu.s x{}, f{}", rd, rs1),
            0x0d => format!("fcvt.lu.d x{}, f{}", rd, rs1),
            0x0f => format!("fcvt.lu.q x{}, f{}", rd, rs1),
            // From signed 32-bit
            0x10 => format!("fcvt.s.w f{}, x{}", rd, rs1),
            0x11 => format!("fcvt.d.w f{}, x{}", rd, rs1),
            0x13 => format!("fcvt.q.w f{}, x{}", rd, rs1),
            // From unsigned 32-bit
            0x14 => format!("fcvt.s.wu f{}, x{}", rd, rs1),
            0x15 => format!("fcvt.d.wu f{}, x{}", rd, rs1),
            0x17 => format!("fcvt.q.wu f{}, x{}", rd, rs1),
            // From signed 64-bit
            0x18 => format!("fcvt.s.l f{}, x{}", rd, rs1),
            0x19 => format!("fcvt.d.l f{}, x{}", rd, rs1),
            0x1b => format!("fcvt.q.l f{}, x{}", rd, rs1),
            // From unsigned 64-bit
            0x1c => format!("fcvt.s.lu f{}, x{}", rd, rs1),
            0x1d => format!("fcvt.d.lu f{}, x{}", rd, rs1),
            0x1f => format!("fcvt.q.lu f{}, x{}", rd, rs1),
            _ => format!("fcvt.??? x{}, x{}, rs2={}", rd, rs1, rs2),
        }
    }

    /// 尝试识别伪指令模式
    /// 
    /// 返回识别到的伪指令字符串，如果不是伪指令则返回 None
    fn try_recognize_pseudo(code: u32, rd: u8, funct3: u8, rs1: u8, rs2: u8, funct7: u8) -> Option<String> {
        let opcode = code & 0x7f;
        
        match opcode {
            // I-type: addi, xori, slti, sltiu, ori, andi
            0x13 => {
                let imm = ((code >> 20) as i32) as i64;
                let imm_u32 = (code >> 20) & 0xfff;
                
                match funct3 {
                    // addi: 检查 nop, mv, li
                    0x0 => {
                        // nop: addi x0, x0, 0
                        if rd == 0 && rs1 == 0 && imm == 0 {
                            return Some("nop".to_string());
                        }
                        // mv rd, rs: addi rd, rs, 0
                        if imm == 0 && rs1 != 0 {
                            return Some(format!("mv x{}, x{}", rd, rs1));
                        }
                        // li rd, imm: addi rd, x0, imm (小立即数)
                        if rs1 == 0 && rd != 0 {
                            return Some(format!("li x{}, {}", rd, imm));
                        }
                    }
                    // xori: 检查 not
                    0x4 => {
                        // not rd, rs: xori rd, rs, -1
                        if imm_u32 == 0xfff {
                            return Some(format!("not x{}, x{}", rd, rs1));
                        }
                    }
                    // sltiu: 检查 seqz
                    0x3 => {
                        // seqz rd, rs: sltiu rd, rs, 1
                        if imm == 1 {
                            return Some(format!("seqz x{}, x{}", rd, rs1));
                        }
                    }
                    _ => {}
                }
            }
            // R-type: add, sub, slt, sltu, etc.
            0x33 => {
                match funct3 {
                    // add/sub: 检查 neg
                    0x0 => {
                        // neg rd, rs: sub rd, x0, rs
                        if (funct7 & 0x20) != 0 && rs1 == 0 {
                            return Some(format!("neg x{}, x{}", rd, rs2));
                        }
                    }
                    // slt: 检查 sltz, sgtz
                    0x2 => {
                        // sltz rd, rs: slt rd, rs, x0
                        if rs2 == 0 {
                            return Some(format!("sltz x{}, x{}", rd, rs1));
                        }
                        // sgtz rd, rs: slt rd, x0, rs
                        if rs1 == 0 {
                            return Some(format!("sgtz x{}, x{}", rd, rs2));
                        }
                    }
                    // sltu: 检查 snez
                    0x3 => {
                        // snez rd, rs: sltu rd, x0, rs
                        if rs1 == 0 && rs2 != 0 {
                            return Some(format!("snez x{}, x{}", rd, rs2));
                        }
                    }
                    _ => {}
                }
            }
            // JAL: 检查 j, jal (省略 rd)
            0x6f => {
                let offset = Self::extract_j_imm(code);
                // j label: jal x0, label
                if rd == 0 {
                    return Some(format!("j {:+}", offset));
                }
                // jal label: jal x1, label (省略 x1)
                if rd == 1 {
                    return Some(format!("jal {:+}", offset));
                }
            }
            // JALR: 检查 jr, jalr, ret
            0x67 => {
                let imm = ((code >> 20) as i32) as i64;
                // ret: jalr x0, x1, 0
                if rd == 0 && rs1 == 1 && imm == 0 {
                    return Some("ret".to_string());
                }
                // jr rs: jalr x0, rs, 0
                if rd == 0 && imm == 0 {
                    return Some(format!("jr x{}", rs1));
                }
                // jalr rs: jalr x1, rs, 0 (省略 x1)
                if rd == 1 && imm == 0 {
                    return Some(format!("jalr x{}", rs1));
                }
            }
            // Branch: 检查 beqz, bnez, blez, bgez, bltz, bgtz
            0x63 => {
                let offset = Self::extract_b_imm(code) as i32;
                match funct3 {
                    // beq: 检查 beqz
                    0x0 => {
                        // beqz rs, label: beq rs, x0, label
                        if rs2 == 0 {
                            return Some(format!("beqz x{}, {:+}", rs1, offset));
                        }
                    }
                    // bne: 检查 bnez
                    0x1 => {
                        // bnez rs, label: bne rs, x0, label
                        if rs2 == 0 {
                            return Some(format!("bnez x{}, {:+}", rs1, offset));
                        }
                    }
                    // blt: 检查 bltz
                    0x4 => {
                        // bltz rs, label: blt rs, x0, label
                        if rs2 == 0 {
                            return Some(format!("bltz x{}, {:+}", rs1, offset));
                        }
                    }
                    // bge: 检查 bgez
                    0x5 => {
                        // bgez rs, label: bge rs, x0, label
                        if rs2 == 0 {
                            return Some(format!("bgez x{}, {:+}", rs1, offset));
                        }
                    }
                    // bltu: 检查 bgtz (blt x0, rs, label)
                    0x6 => {
                        // bgtz rs, label: bltu x0, rs, label (实际上是 blt x0, rs)
                        if rs1 == 0 {
                            return Some(format!("bgtz x{}, {:+}", rs2, offset));
                        }
                    }
                    // bgeu: 检查 blez (bge x0, rs, label 即 ble rs, x0)
                    0x7 => {
                        // blez rs, label: bgeu x0, rs, label (实际上是 bge x0, rs)
                        if rs1 == 0 {
                            return Some(format!("blez x{}, {:+}", rs2, offset));
                        }
                    }
                    _ => {}
                }
            }
            // LUI: 可以是 li 的一部分
            0x37 => {
                // lui rd, upper 可以是 li rd, upper<<12 的简化形式
                // 但通常 lui 单独出现时仍显示为 lui
                // 这里不特殊处理，保持原样
            }
            _ => {}
        }
        
        None
    }

    fn extract_j_imm(code: u32) -> i64 {
        let imm20 = (code >> 31) & 1;
        let imm10_1 = (code >> 21) & 0x3ff;
        let imm11 = (code >> 20) & 1;
        let imm19_12 = (code >> 12) & 0xff;
        
        let imm = (imm20 << 20) | (imm19_12 << 12) | (imm11 << 11) | (imm10_1 << 1);
        
        if imm20 != 0 {
            (imm | 0xfff00000) as i32 as i64
        } else {
            imm as i64
        }
    }

    fn extract_b_imm(code: u32) -> i64 {
        let imm12 = (code >> 31) & 1;
        let imm10_5 = (code >> 25) & 0x3f;
        let imm4_1 = (code >> 8) & 0xf;
        let imm11 = (code >> 7) & 1;
        
        let imm = (imm12 << 12) | (imm11 << 11) | (imm10_5 << 5) | (imm4_1 << 1);
        
        if imm12 != 0 {
            (imm | 0xfffff000) as i32 as i64
        } else {
            imm as i64
        }
    }
}

pub fn assemble_string(source: &str, base_address: u64) -> Result<Vec<u8>, AssemblyError> {
    let mut assembler = Assembler::new();
    assembler.set_base_addresses(base_address, base_address + 0x10000, base_address + 0x20000);
    let lines: Vec<String> = source.lines().map(|s| s.to_string()).collect();
    assembler.assemble(&lines)
}

/// Assembles an assembly file and returns machine code.
///
/// Convenience function that creates an assembler and assembles the file.
///
/// ---
///
/// 汇编汇编文件并返回机器码。
///
/// 便捷函数，创建汇编器并汇编文件。
pub fn assemble_file(filename: &str, base_address: u64) -> Result<Vec<u8>, AssemblyError> {
    let mut assembler = Assembler::new();
    assembler.set_base_addresses(base_address, base_address + 0x10000, base_address + 0x20000);
    assembler.assemble_file(filename)
}

/// Disassembles a single 32-bit instruction.
///
/// Returns a human-readable assembly string.
///
/// ---
///
/// 反汇编单条 32 位指令。
///
/// 返回可读的汇编字符串。
pub fn disassemble_instruction(code: u32) -> String {
    let assembler = Assembler::new();
    assembler.disassemble(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nop() {
        // addi x0, x0, 0
        let code: u32 = 0x00000013;
        assert_eq!(disassemble_instruction(code), "nop");
    }

    #[test]
    fn test_mv() {
        // addi x10, x11, 0
        let code: u32 = 0x00058513;
        assert_eq!(disassemble_instruction(code), "mv x10, x11");
    }

    #[test]
    fn test_li_small() {
        // addi x10, x0, 42
        let code: u32 = 0x02a00513;
        assert_eq!(disassemble_instruction(code), "li x10, 42");
    }

    #[test]
    fn test_li_zero() {
        // addi x10, x0, 0
        let code: u32 = 0x00000513;
        assert_eq!(disassemble_instruction(code), "li x10, 0");
    }

    #[test]
    fn test_not() {
        // xori x10, x11, -1 (0xfff)
        let code: u32 = 0xfff5c513;
        assert_eq!(disassemble_instruction(code), "not x10, x11");
    }

    #[test]
    fn test_neg() {
        // sub x10, x0, x11
        let code: u32 = 0x40b00533;
        assert_eq!(disassemble_instruction(code), "neg x10, x11");
    }

    #[test]
    fn test_seqz() {
        // sltiu x10, x11, 1
        let code: u32 = 0x0015b513;
        assert_eq!(disassemble_instruction(code), "seqz x10, x11");
    }

    #[test]
    fn test_snez() {
        // sltu x10, x0, x11
        let code: u32 = 0x00b03533;
        assert_eq!(disassemble_instruction(code), "snez x10, x11");
    }

    #[test]
    fn test_sltz() {
        // slt x10, x11, x0
        let code: u32 = 0x0005a533;
        assert_eq!(disassemble_instruction(code), "sltz x10, x11");
    }

    #[test]
    fn test_sgtz() {
        // slt x10, x0, x11
        let code: u32 = 0x00b02533;
        assert_eq!(disassemble_instruction(code), "sgtz x10, x11");
    }

    #[test]
    fn test_ret() {
        // jalr x0, x1, 0
        let code: u32 = 0x00008067;
        assert_eq!(disassemble_instruction(code), "ret");
    }

    #[test]
    fn test_jr() {
        // jalr x0, x10, 0
        let code: u32 = 0x00050067;
        assert_eq!(disassemble_instruction(code), "jr x10");
    }

    #[test]
    fn test_jalr_pseudo() {
        // jalr x1, x10, 0 -> jalr x10 (pseudo)
        // I-type: imm[11:0] | rs1 | funct3 | rd | opcode
        // 验证编码:
        // rd=1, rs1=10, imm=0, opcode=0x67, funct3=0
        // = (0 << 20) | (10 << 15) | (0 << 12) | (1 << 7) | 0x67
        let code: u32 = (0u32 << 20) | (10u32 << 15) | (0u32 << 12) | (1u32 << 7) | 0x67u32;
        println!("jalr_pseudo test: code=0x{:08x}", code);
        assert_eq!(disassemble_instruction(code), "jalr x10");
    }

    #[test]
    fn test_j() {
        // jal x0, offset
        let code: u32 = 0x0040006f; // jal x0, +4
        let result = disassemble_instruction(code);
        assert!(result.starts_with("j "), "actual result: {}", result);
    }

    #[test]
    fn test_jal_pseudo() {
        // jal x1, offset
        let code: u32 = 0x004000ef; // jal x1, +4
        let result = disassemble_instruction(code);
        assert!(result.starts_with("jal "), "actual result: {}", result);
    }
}
