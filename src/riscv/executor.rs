use crate::riscv::instruction;
use crate::riscv::memory;
use crate::riscv::register;
use crate::riscv::cpu;

pub struct ExecutionResult {
    success: bool,
    branch_taken: bool,
    next_pc: memory::Address128,
    error_message: String,
}

pub struct Executor {}

impl ExecutionResult {
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
    fn get_reg_value(regs: &register::Register, reg: u8) -> i128 {
        if reg == 0 {
            return 0;
        }
        if reg > 31 {
            panic!("Invalid register index: {}", reg);
        }
        regs.read(unsafe { std::mem::transmute::<u8, register::RegisterIndex>(reg) }) as i128
    }

    fn set_reg_value(regs: &mut register::Register, reg: u8, value: i128) {
        if reg == 0 {
            return;
        }
        if reg > 31 {
            panic!("Invalid register index: {}", reg);
        }
        regs.write(unsafe { std::mem::transmute::<u8, register::RegisterIndex>(reg) }, value as u128);
    }

    fn execute_add(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers_mut();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1);
        let rs2_val = Self::get_reg_value(regs, decoded.rs2);
        let rd_val = rs1_val + rs2_val;
        Self::set_reg_value(regs, decoded.rd, rd_val);
        result
    }

    fn execute_sub(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers_mut();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1);
        let rs2_val = Self::get_reg_value(regs, decoded.rs2);
        let rd_val = rs1_val - rs2_val;
        Self::set_reg_value(regs, decoded.rd, rd_val);
        result
    }

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

    fn execute_slt(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers_mut();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1);
        let rs2_val = Self::get_reg_value(regs, decoded.rs2);
        let rd_val = if rs1_val < rs2_val { 1 } else { 0 };
        Self::set_reg_value(regs, decoded.rd, rd_val);
        result
    }

    fn execute_sltu(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers_mut();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1) as u128;
        let rs2_val = Self::get_reg_value(regs, decoded.rs2) as u128;
        let rd_val = if rs1_val < rs2_val { 1 } else { 0 };
        Self::set_reg_value(regs, decoded.rd, rd_val);
        result
    }

    fn execute_xor(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers_mut();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1);
        let rs2_val = Self::get_reg_value(regs, decoded.rs2);
        let rd_val = rs1_val ^ rs2_val;
        Self::set_reg_value(regs, decoded.rd, rd_val);
        result
    }

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

    fn execute_or(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers_mut();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1);
        let rs2_val = Self::get_reg_value(regs, decoded.rs2);
        let rd_val = rs1_val | rs2_val;
        Self::set_reg_value(regs, decoded.rd, rd_val);
        result
    }
}