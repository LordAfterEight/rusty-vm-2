#[repr(u32)]
#[derive(Display, num_enum::TryFromPrimitive, Debug, PartialEq)]
// 0x00 - 0x7F
#[allow(non_camel_case_types)]
pub enum OpCode {
    /// OP - xxx
    NOOP = 0x00,

    /// OP(7) - RDE(5) - IMM(20)
    LOAD_IMM = 0x01,

    /// OP(7) - RS1(5) - IMM(20)
    STOR_IMM = 0x02,

    /// OP(7) - IMM(25)
    JUMP_IMM = 0x10,

    /// OP(7) - RS1(5) - xxx
    JUMP_REG = 0x11,

    /// OP(7) - IMM(25)
    BRAN_IMM = 0x30,

    /// OP(7) - RS1(5) - xxx
    BRAN_REG = 0x31,

    /// OP(7) - xxx
    RTRN = 0x3E,

    /// OP(7) - xxx
    RTRN_POP = 0x3D,

    /// OP(7) - RDE(5) - RS1(5) - RS2(5) - MOD(10)
    ADD = 0x20,

    /// OP(7) - RDE(5) - RS1(5) - RS2(5) - MOD(10)
    SUB = 0x21,

    /// OP(7) - RDE(5) - RS1(5) - RS2(5) - MOD(10)
    MUL = 0x22,

    /// OP(7) - RDE(5) - RS1(5) - RS2(5) - MOD(10)
    DIV = 0x23,

    /// OP(7) - RDE(5) - RS1(5) - RS2(5) - MOD(10)
    AND = 0x24,

    /// OP(7) - RDE(5) - RS1(5) - RS2(5) - MOD(10)
    OR = 0x25,

    /// OP(7) - xxx
    RSET_SOFT = 0x40,

    /// OP(7) - xxx
    RSET_HARD = 0x41,

    /// OP(7) - xxx
    HALT = 0x4F,
}
