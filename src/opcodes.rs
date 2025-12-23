#[repr(u32)]
#[derive(Display, num_enum::TryFromPrimitive, Debug, PartialEq)]
// 0x00 - 0x7F
#[allow(non_camel_case_types)]
pub enum OpCode {
    /// OP - xxx
    NOOP = 0x00,

    /// OP(7) - RDE(5) - IMM(20)
    /// Loads an immediate 20-bit value to register RDE
    LOAD_IMM = 0x01,

    /// OP(7) - RDE(5) - IMM(20)
    /// Loads an immediate 20-bit value to the upper 20 bits of register RDE
    LDUP_IMM = 0x02,

    /// OP(7) - RS1(5) - IMM(20)
    /// Writes the value of register RS1 to the immediate 20-bit address
    STOR_IMM = 0x03,

    /// OP(7) - RDE(5) - RS1(5) - xxx
    /// Loads a byte from the address stored in register RS1 to RDE
    LOAD_BYTE = 0x04,

    /// OP(7) - RS1(5) - RS2(5) - xxx
    /// Writes the value from register RS1 to the address stored in register RS2
    STOR_BYTE = 0x05,

    /// OP(7) - IMM(25)
    /// Unconditionally jumps to the immediate 25-bit address
    JUMP_IMM = 0x10,

    /// OP(7) - RS1(5) - xxx
    /// Unconditionally jumps to the address stored in register RS1
    JUMP_REG = 0x11,

    /// OP(7) - IMM(25)
    /// Unconditionally branches to the immediate 25-bit address
    BRAN_IMM = 0x12,

    /// OP(7) - RS1(5) - xxx
    /// Unconditionally branches to the address stored in register RS1
    BRAN_REG = 0x13,

    /// OP(7) - RDE(5) - RS1(5) - RS2(5) - xxx
    /// Adds the contents of registers RS1 and RS2 and stores the result in register RS2
    ADD = 0x20,

    /// OP(7) - RDE(5) - RS1(5) - RS2(5) - xxx
    /// Subtracts the contents of registers RS1 and RS2 and stores the result in register RS2
    SUB = 0x21,

    /// OP(7) - RDE(5) - RS1(5) - RS2(5) - xxx
    AND = 0x24,

    /// OP(7) - RDE(5) - RS1(5) - RS2(5) - xxx
    /// ORs the content or register RS1 and RS2, storing the result to register RDE
    ORR = 0x25,

    /// OP(7) - RDE(5) - IMM(20)
    /// ORs the content of register RDE with the 20-bit immediate value
    ORI = 0x26,

    /// OP(7) - RDE(5) - RS1(5) - RS2(5) - xxx
    XOR = 0x27,

    /// OP(7) - xxx
    RTRN = 0x3E,

    /// OP(7) - xxx
    RTRN_POP = 0x3D,

    /// OP(7) - xxx
    RSET_SOFT = 0x40,

    /// OP(7) - xxx
    RSET_HARD = 0x41,

    /// OP(7) - xxx
    HALT = 0x4F,

    /// OP(7) - 
    IRPT_SEND = 0x50,
}
