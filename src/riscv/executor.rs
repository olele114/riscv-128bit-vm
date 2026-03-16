use crate::riscv::instruction;
use crate::riscv::memory;
use crate::riscv::register;
use crate::riscv::cpu;
use crate::riscv::memory::Memory;

pub struct ExecutionResult {
    pub(crate) success: bool,
    pub(crate) branch_taken: bool,
    pub(crate) next_pc: memory::Address128,
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

    // R-type
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

    fn execute_and(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers_mut();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1);
        let rs2_val = Self::get_reg_value(regs, decoded.rs2);
        let rd_val = rs1_val & rs2_val;
        Self::set_reg_value(regs, decoded.rd, rd_val);
        result
    }

    // I-type
    fn execute_addi(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers_mut();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1);
        let rd_val = rs1_val + decoded.imm;
        Self::set_reg_value(regs, decoded.rd, rd_val);
        result
    }

    fn execute_slli(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers_mut();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1);
        let shift = (decoded.imm & 0x7f) as u128;
        let rd_val = rs1_val << shift;
        Self::set_reg_value(regs, decoded.rd, rd_val);
        result
    }

    fn execute_slti(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers_mut();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1);
        let imm_val = decoded.imm;
        let rd_val = if rs1_val < imm_val { 1 } else { 0 };
        Self::set_reg_value(regs, decoded.rd, rd_val);
        result
    }

    fn execute_sltiu(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers_mut();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1) as u128;
        let imm_val = decoded.imm as u128;
        let rd_val = if rs1_val < imm_val { 1 } else { 0 };
        Self::set_reg_value(regs, decoded.rd, rd_val);
        result
    }

    fn execute_xori(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers_mut();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1);
        let rd_val = rs1_val ^ decoded.imm;
        Self::set_reg_value(regs, decoded.rd, rd_val);
        result
    }

    fn execute_srli(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers_mut();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1) as u128;
        let shift = (decoded.imm & 0x7f) as u128;
        let rd_val = (rs1_val >> shift) as i128;
        Self::set_reg_value(regs, decoded.rd, rd_val);
        result
    }

    fn execute_srai(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers_mut();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1);
        let shift = (decoded.imm & 0x7f) as u128;
        let rd_val = rs1_val >> shift;
        Self::set_reg_value(regs, decoded.rd, rd_val);
        result
    }

    fn execute_ori(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers_mut();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1);
        let rd_val = rs1_val | decoded.imm;
        Self::set_reg_value(regs, decoded.rd, rd_val);
        result
    }

    fn execute_andi(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let regs = cpu.get_registers_mut();
        let rs1_val = Self::get_reg_value(regs, decoded.rs1);
        let rd_val = rs1_val & decoded.imm;
        Self::set_reg_value(regs, decoded.rd, rd_val);
        result
    }

    // Load instructions

    fn execute_lb(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let rs1_val = Self::get_reg_value(cpu.get_registers(), decoded.rs1);
        let addr = (rs1_val + decoded.imm) as memory::Address128;
        let rd_val = cpu.get_memory().read_8(addr);
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, rd_val as i128);
        result
    }

    fn execute_lh(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let rs1_val = Self::get_reg_value(cpu.get_registers(), decoded.rs1);
        let addr = (rs1_val + decoded.imm) as memory::Address128;
        let rd_val = cpu.get_memory().read_16(addr);
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, rd_val as i128);
        result
    }

    fn execute_lw(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let rs1_val = Self::get_reg_value(cpu.get_registers(), decoded.rs1);
        let addr = (rs1_val + decoded.imm) as memory::Address128;
        let rd_val = cpu.get_memory().read_32(addr);
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, rd_val as i128);
        result
    }

    fn execute_ld(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let rs1_val = Self::get_reg_value(cpu.get_registers(), decoded.rs1);
        let addr = (rs1_val + decoded.imm) as memory::Address128;
        let rd_val = cpu.get_memory().read_64(addr);
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, rd_val as i128);
        result
    }

    fn execute_lq(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let rs1_val = Self::get_reg_value(cpu.get_registers(), decoded.rs1);
        let addr = (rs1_val + decoded.imm) as memory::Address128;
        let rd_val = cpu.get_memory().read_128(addr);
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, rd_val as i128);
        result
    }

    fn execute_lbu(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let rs1_val = Self::get_reg_value(cpu.get_registers(), decoded.rs1);
        let addr = (rs1_val + decoded.imm) as memory::Address128;
        let rd_val = cpu.get_memory().read_8(addr) as u128;
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, rd_val as i128);
        result
    }

    fn execute_lhu(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let rs1_val = Self::get_reg_value(cpu.get_registers(), decoded.rs1);
        let addr = (rs1_val + decoded.imm) as memory::Address128;
        let rd_val = cpu.get_memory().read_16(addr) as u128;
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, rd_val as i128);
        result
    }

    fn execute_lwu(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let rs1_val = Self::get_reg_value(cpu.get_registers(), decoded.rs1);
        let addr = (rs1_val + decoded.imm) as memory::Address128;
        let rd_val = cpu.get_memory().read_32(addr) as u128;
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, rd_val as i128);
        result
    }

    fn execute_ldu(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let rs1_val = Self::get_reg_value(cpu.get_registers(), decoded.rs1);
        let addr = (rs1_val + decoded.imm) as memory::Address128;
        let rd_val = cpu.get_memory().read_64(addr) as u128;
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, rd_val as i128);
        result
    }

    // Store instructions
    fn execute_sb(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let rs1_val = Self::get_reg_value(cpu.get_registers(), decoded.rs1);
        let rs2_val = Self::get_reg_value(cpu.get_registers(), decoded.rs2);
        let addr = (rs1_val + decoded.imm) as memory::Address128;
        cpu.get_memory_mut().write_8(addr, rs2_val as u8);
        result
    }

    fn execute_sh(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let rs1_val = Self::get_reg_value(cpu.get_registers(), decoded.rs1);
        let rs2_val = Self::get_reg_value(cpu.get_registers(), decoded.rs2);
        let addr = (rs1_val + decoded.imm) as memory::Address128;
        cpu.get_memory_mut().write_16(addr, rs2_val as u16);
        result
    }

    fn execute_sw(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let rs1_val = Self::get_reg_value(cpu.get_registers(), decoded.rs1);
        let rs2_val = Self::get_reg_value(cpu.get_registers(), decoded.rs2);
        let addr = (rs1_val + decoded.imm) as memory::Address128;
        cpu.get_memory_mut().write_32(addr, rs2_val as u32);
        result
    }

    fn execute_sd(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let rs1_val = Self::get_reg_value(cpu.get_registers(), decoded.rs1);
        let rs2_val = Self::get_reg_value(cpu.get_registers(), decoded.rs2);
        let addr = (rs1_val + decoded.imm) as memory::Address128;
        cpu.get_memory_mut().write_64(addr, rs2_val as u64);
        result
    }

    fn execute_sq(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let rs1_val = Self::get_reg_value(cpu.get_registers(), decoded.rs1);
        let rs2_val = Self::get_reg_value(cpu.get_registers(), decoded.rs2);
        let addr = (rs1_val + decoded.imm) as memory::Address128;
        cpu.get_memory_mut().write_128(addr, rs2_val as memory::Word128);
        result
    }

    // Branch instructions
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

    // U-type instructions
    fn execute_lui(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, decoded.imm);
        result
    }

    fn execute_auipc(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let result = ExecutionResult::new();
        let pc = cpu.get_registers().get_pc();
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, pc as i128 + decoded.imm);
        result
    }

    // J-type instructions
    fn execute_jal(cpu: &mut cpu::CPU, decoded: &instruction::DecodedInstruction) -> ExecutionResult {
        let mut result = ExecutionResult::new();

        let pc = cpu.get_registers().get_pc();
        Self::set_reg_value(cpu.get_registers_mut(), decoded.rd, pc as i128 + 4);

        result.branch_taken = true;
        result.next_pc = (cpu.get_registers().get_pc() as i128 + decoded.imm) as memory::Address128;
        result
    }

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

    // System instructions
    fn execute_ecall(cpu: &mut cpu::CPU) -> ExecutionResult {
        let mut result = ExecutionResult::new();
        cpu.raise_exception(cpu::ExceptionType::SystemCall, 0, cpu.get_registers().get_pc(), String::from("ECALL"));
        result.success = false;
        result.error_message = String::from("System call");
        result
    }

    fn execute_ebreak(cpu: &mut cpu::CPU) -> ExecutionResult {
        let mut result = ExecutionResult::new();
        cpu.raise_exception(cpu::ExceptionType::Breakpoint, 0, cpu.get_registers().get_pc(), String::from("EBREAK"));
        result.success = false;
        result.error_message = String::from("Breakpoint");
        result
    }
}