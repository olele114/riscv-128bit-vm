#[repr(u8)]
enum OpCode {
    Lui = 0x37,
    Auipc = 0x17,
    Jal = 0x6f,
    Jalr = 0x67,
    Branch = 0x63,
    Load = 0x03,
    Store = 0x23,
    Imm = 0x13,
    Reg = 0x33,
    System = 0x73,
    MiscMem = 0x0f,
}

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

#[repr(u8)]
enum Funct7 {
    None = 0x00,
    SubSra = 0x20,
    MulDiv = 0x01,
}

#[repr(u8)]
enum BranchFunct3 {
    Beq = 0x0,
    Bne = 0x1,
    Blt = 0x4,
    Bge = 0x5,
    Bltu = 0x6,
    Bgeu = 0x7,
}

#[repr(u8)]
enum LoadFunct3 {
    Lb = 0x0,
    Lh = 0x1,
    Lw = 0x2,
    Ld = 0x3,
    Lq = 0x4,
    Lhu = 0x5,
    Lwu = 0x6,
    Ldu = 0x7,
}

#[repr(u8)]
enum StoreFunct3 {
    Sb = 0x0,
    Sh = 0x1,
    Sw = 0x2,
    Sd = 0x3,
    Sq = 0x4,
}

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

enum InstructionType {
    RType,
    IType,
    SType,
    BType,
    UType,
    JType,
    Unknown,
}

struct DecodedInstruction {
    opcode: OpCode,
    typ: InstructionType,
    rd: u8,
    rs1: u8,
    rs2: u8,
    funct3: u8,
    funct7: u8,
    imm: i128,
}

struct InstructionDecoder {}

impl DecodedInstruction {
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
    pub fn extract_opcode(instruction: u32) -> u8 {
        return (instruction & 0x7f) as u8;
    }

    pub fn extract_rd(instruction: u32) -> u8 {
        return ((instruction >> 7) & 0x1f) as u8;
    }

    pub fn extract_funct3(instruction: u32) -> u8 {
        return ((instruction >> 12) & 0x7) as u8;
    }

    pub fn extract_rs1(instruction: u32) -> u8 {
        return ((instruction >> 15) & 0x1f) as u8;
    }

    pub fn extract_rs2(instruction: u32) -> u8 {
        return ((instruction >> 20) & 0x1f) as u8;
    }

    pub fn extract_funct7(instruction: u32) -> u8 {
        return ((instruction >> 25) & 0x7f) as u8;
    }

    pub fn sign_extend(value: u64, bits: u8) -> i128 {
        let mask = 1 << (bits - 1);
        if value & mask != 0 {
            let extension = !((1 << bits) - 1);
            return (value | extension) as i128;
        }
        value as i128
    }

    pub fn extract_imm_i(instruction: u32) -> i128 {
        let imm = ((instruction >> 20) & 0xfff);
        Self::sign_extend(imm as u64, 12)
    }

    pub fn extract_imm_s(instruction: u32) -> i128 {
        let imm = ((instruction >> 7) & 0x1f) | (((instruction >> 25) & 0x7f) << 5);
        Self::sign_extend(imm as u64, 12)
    }

    pub fn extract_imm_b(instruction: u32) -> i128 {
        let imm = (((instruction >> 8) & 0xf) << 1) |
            (((instruction >> 25) & 0x3f) << 5) |
            (((instruction >> 7) & 0x1) << 11) |
            (((instruction >> 31) & 0x1) << 12);
        Self::sign_extend(imm as u64, 13)
    }

    pub fn extract_imm_u(instruction: u32) -> i128 {
        (instruction & 0xfffff000) as i128
    }

    pub fn extract_imm_j(instruction: u32) -> i128 {
        let imm = (((instruction >> 21) & 0x3ff) << 1) |
            (((instruction >> 20) & 0x1) << 11) |
            (((instruction >> 12) & 0xff) << 12) |
            (((instruction >> 31) & 0x1) << 20);
        Self::sign_extend(imm as u64, 21)
    }

    pub fn get_instruction_type(op_code: &OpCode, funct3: u8) -> InstructionType {
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
                if (decoded.funct3 == 0x0) {
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