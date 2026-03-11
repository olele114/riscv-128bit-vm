type Register128 = u128;
type Register64 = u64;

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

pub struct SpecialRegisters {
    pc: Register128,
    fcsr: Register64,
}

pub struct Register {
    registers: [Register128; 32],
    special: SpecialRegisters,
}

impl Register {
    pub fn new() -> Self {
        let mut tmp = Self {
            registers: [0; 32],
            special: SpecialRegisters {
                pc: 0,
                fcsr: 0,
            },
        };
        tmp.reset();
        tmp
    }

    pub fn read(&self, index: RegisterIndex) -> Register128 {
        if (index > RegisterIndex::X31) {
            panic!("Invalid register index: Index out of range");
        }
        self.registers[index as usize]
    }

    pub fn write(&mut self, index: RegisterIndex, value: Register128) {
        if (index > RegisterIndex::X31) {
            panic!("Invalid register index: Index out of range");
        }
        if (index != RegisterIndex::X0) {
            self.registers[index as usize] = value;
        }
    }

    pub fn get_pc(&self) -> Register128 {
        self.special.pc
    }

    pub fn set_pc(&mut self, value: Register128) {
        self.special.pc = value;
    }

    pub fn reset(&mut self) {
        self.registers = [0; 32];
        self.special = SpecialRegisters {
            pc: 0,
            fcsr: 0,
        };
    }

    pub fn get_special_registers(&self) -> &SpecialRegisters {
        &self.special
    }

    pub fn get_special_registers_mut(&mut self) -> &mut SpecialRegisters {
        &mut self.special
    }
}