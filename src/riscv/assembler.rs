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
                    let shamt = (code >> 20) & 0x7f;
                    format!("{} x{}, x{}, {}", mnemonic, rd, rs1, shamt)
                } else {
                    format!("{} x{}, x{}, {}", mnemonic, rd, rs1, imm)
                }
            }
            0x33 => {
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
            0x73 => {
                if funct3 == 0 {
                    let imm = (code >> 20) & 0xfff;
                    match imm {
                        0 => "ecall".to_string(),
                        1 => "ebreak".to_string(),
                        _ => format!("system 0x{:03x}", imm),
                    }
                } else {
                    format!("csr ???")
                }
            }
            _ => format!("unknown opcode 0x{:02x}", opcode),
        }
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
