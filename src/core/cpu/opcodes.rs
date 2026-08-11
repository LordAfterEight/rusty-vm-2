#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(clippy::upper_case_acronyms)]
#[repr(u32)]
pub enum OpCode {
    // Loading OpCodes
    LDIV = 0x10, // Load immediate 20-bit value to rd
    LDRE = 0x11, // Load value from rs1 to rd
    LD08 = 0x12, // Load byte from address in rs1 to rd
    LD16 = 0x13, // Load halfword from address in rs1 to rd
    LD32 = 0x14, // Load word from address in rs1 to rd
    LDHW = 0x15, // Load an immediate 16-bit value to rd
    LDWD = 0x16, // Load an immediate 32-bit value to rd

    // Storing OpCodes
    STIV = 0x20, // Store immediate 20-bit value to address in rs1
    STIA = 0x21, // Store value from rs1 to immediate 20-bit address
    ST08 = 0x22, // Stores the first 8 bits of rs1 to address in rs2
    ST16 = 0x23, // Stores the first 16 bits of rs1 to address in rs2
    ST32 = 0x24, // Stores all 32 bits of rs1 to address in rs2

    // Jumping OpCodes
    JMIM = 0x30, // Jump unconditionally to an immediate 25-bit address
    JMRE = 0x31, // Jump unconditionally to the address in rs1
    JMRL = 0x32, // Jump unconditionally and relative to current position, immediate is a signed 25-bit value
    JMIZ = 0x33, // Same as JMRL but only jump with a 20-bit address and if rs1 is zero
    JMNZ = 0x34, // Same as JMRL but only jump with a 20-bit address and if rs1 is NOT zero

    // Branching OpCodes
    // All the same as the JMxx variants, but also push the PC to the stack
    BRIM = 0x35,
    BRRE = 0x36,
    BRRL = 0x37,
    BRIZ = 0x38,
    BRNZ = 0x39,

    // Arithmetic OpCodes
    AADD = 0x40, // Adds rs1 and rs2, stores result in rd
    ASUB = 0x41, // Subtracts rs2 from rs1, stores result in rd
    AMUL = 0x42, // Multiplies rs1 and rs2, stores result in rd

    // Logic OpCodes
    LAND = 0x50, // ANDs rs1 and rs2, stores result in rd
    LORI = 0x51, // ORs rd with an immediate 20-bit value
    LORR = 0x52, // ORs rs1 and rs2, stores result in rd
    LXOR = 0x53, // XORs rs1 and rs2, stores result in rd
    LROR = 0x54, // Shifts content of rs1 right by rs2 bits
    LROL = 0x55, // Shifts content of rs1 left by rs2 bits

    // System OpCodes
    HALT = 0x70, // Halts the CPU
    PUSH = 0x71, // Pushes value from rs1 to the stack, advancing the stack pointer
    PULL = 0x72, // Pulls value from the stack to rd, retreating the stack pointer
    IRPT = 0x73, // ! TODO
    SRES = 0x74, // Soft reset the CPU
    HRES = 0x75, // Same as SRES, also clears the registers and state
    RTRN = 0x76, // Pull PC from stack and jump to it
    RTIZ = 0x77, // Same as RTRN, but only executes if zero flag is set
    RTNZ = 0x78, // Same as RTRN, but only executes if zero flag is NOT set
}

#[derive(PartialEq, Eq, Debug)]
pub enum DecodingError {
    InvalidOpCode(u8),
}

#[derive(Debug)]
pub struct Decoded {
    pub raw: u32,
    pub op: OpCode,
    pub rd: usize,
    pub rs1: usize,
    pub rs2: usize,
    pub imm: u32,
}

impl std::fmt::Display for Decoded {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Raw value: {}", self.raw)
    }
}

impl std::error::Error for Decoded {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }
    fn description(&self) -> &str {
        "Failed to deconstruct a u32 value"
    }
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl TryFrom<u32> for Decoded {
    type Error = DecodingError;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        let opcode = (value >> 25) as u8;
        let op = match opcode {
            0x10 => OpCode::LDIV,
            0x11 => OpCode::LDRE,
            0x12 => OpCode::LD08,
            0x13 => OpCode::LD16,
            0x14 => OpCode::LD32,
            0x15 => OpCode::LDHW,
            0x16 => OpCode::LDWD,
            0x20 => OpCode::STIV,
            0x21 => OpCode::STIA,
            0x22 => OpCode::ST08,
            0x23 => OpCode::ST16,
            0x24 => OpCode::ST32,
            0x30 => OpCode::JMIM,
            0x31 => OpCode::JMRE,
            0x32 => OpCode::JMRL,
            0x33 => OpCode::JMIZ,
            0x34 => OpCode::JMNZ,
            0x35 => OpCode::BRIM,
            0x36 => OpCode::BRRE,
            0x37 => OpCode::BRRL,
            0x38 => OpCode::BRIZ,
            0x39 => OpCode::BRNZ,
            0x40 => OpCode::AADD,
            0x41 => OpCode::ASUB,
            0x42 => OpCode::AMUL,
            0x50 => OpCode::LAND,
            0x51 => OpCode::LORI,
            0x52 => OpCode::LORR,
            0x53 => OpCode::LXOR,
            0x54 => OpCode::LROR,
            0x55 => OpCode::LROL,
            0x70 => OpCode::HALT,
            0x71 => OpCode::PUSH,
            0x72 => OpCode::PULL,
            0x73 => OpCode::IRPT,
            0x74 => OpCode::SRES,
            0x75 => OpCode::HRES,
            0x76 => OpCode::RTRN,
            0x77 => OpCode::RTIZ,
            0x78 => OpCode::RTNZ,
            _ => return Err(DecodingError::InvalidOpCode(opcode)),
        };
        let r1 = ((value >> 20) & 0x1f) as usize;
        let r2 = ((value >> 15) & 0x1f) as usize;
        let r3 = ((value >> 10) & 0x1f) as usize;
        let (rd, rs1, rs2, imm) = match op {
            OpCode::LDIV | OpCode::LORI => (r1, 0, 0, value & 0xfffff),
            OpCode::LDRE | OpCode::LD08 | OpCode::LD16 | OpCode::LD32 => (r1, r2, 0, 0),
            OpCode::LDHW => (r1, 0, 0, value & 0xffff),
            OpCode::LDWD => (r1, 0, 0, 0),
            OpCode::STIV
            | OpCode::STIA
            | OpCode::JMIZ
            | OpCode::JMNZ
            | OpCode::BRIZ
            | OpCode::BRNZ => (0, r1, 0, value & 0xfffff),
            OpCode::JMIM | OpCode::JMRL | OpCode::BRIM | OpCode::BRRL => {
                (0, 0, 0, value & 0x1ffffff)
            }
            OpCode::JMRE | OpCode::BRRE | OpCode::PUSH => (0, r1, 0, 0),
            OpCode::ST08 | OpCode::ST16 | OpCode::ST32 | OpCode::LROR | OpCode::LROL => {
                (0, r1, r2, 0)
            }
            OpCode::AADD
            | OpCode::ASUB
            | OpCode::AMUL
            | OpCode::LAND
            | OpCode::LORR
            | OpCode::LXOR => (r1, r2, r3, 0),
            OpCode::PULL => (r1, 0, 0, 0),
            OpCode::HALT
            | OpCode::IRPT
            | OpCode::SRES
            | OpCode::HRES
            | OpCode::RTRN
            | OpCode::RTIZ
            | OpCode::RTNZ => (0, 0, 0, 0),
        };
        Ok(Self {
            raw: value,
            op,
            rd,
            rs1,
            rs2,
            imm,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(value: u32, op: OpCode, rd: usize, rs1: usize, rs2: usize, imm: u32) {
        let decoded = Decoded::try_from(value).unwrap();
        assert_eq!(
            (
                decoded.op,
                decoded.rd,
                decoded.rs1,
                decoded.rs2,
                decoded.imm
            ),
            (op, rd, rs1, rs2, imm)
        );
    }

    #[test]
    fn decodes_layouts() {
        check(
            0x10 << 25 | 31 << 20 | 0xabcde,
            OpCode::LDIV,
            31,
            0,
            0,
            0xabcde,
        );
        check(0x11 << 25 | 30 << 20 | 29 << 15, OpCode::LDRE, 30, 29, 0, 0);
        check(
            0x40 << 25 | 28 << 20 | 27 << 15 | 26 << 10,
            OpCode::AADD,
            28,
            27,
            26,
            0,
        );
        check(
            0x20 << 25 | 25 << 20 | 0x54321,
            OpCode::STIV,
            0,
            25,
            0,
            0x54321,
        );
        check(0x30 << 25 | 0x1abcdef, OpCode::JMIM, 0, 0, 0, 0x1abcdef);
        check(0x22 << 25 | 24 << 20 | 23 << 15, OpCode::ST08, 0, 24, 23, 0);
    }

    #[test]
    fn decodes_special_layouts() {
        check(
            0x15 << 25 | 22 << 20 | 0xf << 16 | 0xbeef,
            OpCode::LDHW,
            22,
            0,
            0,
            0xbeef,
        );
        check(0x16 << 25 | 21 << 20 | 0xfffff, OpCode::LDWD, 21, 0, 0, 0);
        check(
            0x51 << 25 | 20 << 20 | 0x12345,
            OpCode::LORI,
            20,
            0,
            0,
            0x12345,
        );
        check(0x70 << 25 | 0x1ffffff, OpCode::HALT, 0, 0, 0, 0);
    }

    #[test]
    fn rejects_invalid_opcode() {
        assert_eq!(
            Decoded::try_from(0x7f << 25).unwrap_err(),
            DecodingError::InvalidOpCode(0x7f)
        );
    }
}
