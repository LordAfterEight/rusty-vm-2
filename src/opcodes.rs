#[repr(u32)]
#[derive(Display, num_enum::TryFromPrimitive, Debug, PartialEq)]
// 0x00 - 0x7F
#[allow(non_camel_case_types)]
pub enum OpCode {
    /// OP - xxx
    NOOP = 0x00,

    /// OP(7) - RDE(5) - IMM(20)
    /// Loads an immediate 20-bit value to register RDE.
    LOAD_IMM = 0x01,

    /// OP(7) - RDE(5) - IMM(20)
    /// Loads an immediate 20-bit value to the upper 20 bits of register RDE.
    LDUP_IMM = 0x02,

    /// OP(7) - RS1(5) - IMM(20)
    /// Writes the value of register RS1 to the immediate 20-bit address.
    STOR_IMM = 0x03,

    /// OP(7) - RDE(5) - RS1(5) - xxx
    /// Loads a byte from the address stored in register RS1 to RDE.
    LOAD_BYTE = 0x04,

    /// OP(7) - RS1(5) - RS2(5) - xxx
    /// Writes the value from register RS1 to the address stored in register RS2.
    STOR_BYTE = 0x05,

    /// OP(7) - IMM(25)
    /// Unconditionally jumps to the immediate 25-bit address.
    JUMP_IMM = 0x10,

    /// OP(7) - RS1(5) - xxx
    /// Unconditionally jumps to the address stored in register RS1.
    JUMP_REG = 0x11,

    /// OP(7) - IMM(25)
    /// Unconditionally branches to the immediate 25-bit address. Writes the current position
    /// to the address the stack pointer is pointing to before jumping.
    BRAN_IMM = 0x12,

    /// OP(7) - RS1(5) - xxx
    /// Unconditionally branches to the address stored in register RS1. Writes the current position
    /// to the address the stack pointer is pointing to before jumping.
    BRAN_REG = 0x13,

    /// OP(7) - RS1(5) - RS2(5) - RS3(5)
    /// Compares registers RS1 and RS2, jumps to the address stored in register RS3 if equal.
    JUEQ_REG = 0x14,

    /// OP(7) - RS1(5) - RS2(5) - RS3(5)
    /// Compares registers RS1 and RS2, branches to the address stored in register RS3 if equal. Writes the current position
    /// to the address the stack pointer is pointing to before jumping.
    BREQ_REG = 0x15,

    /// OP(7) - SIG(1) - IMM(19) - xxx
    /// Adds IMM to the program counter. SIG is a sign bit that determines whether IMM is negative (0) or positive(1). This
    /// jump is unconditional.
    JUMP_REL = 0x16,

    /// OP(7) - SIG(1) - IMM(19) - xxx
    /// Adds IMM to the program counter. SIG is a sign bit that determines whether IMM is negative (0) or positive(1). This
    /// branch is unconditional. Writes the current position to the address the stack pointer is pointing to before branching.
    BRAN_REL = 0x17,

    /// OP(7) - RDE(5) - RS1(5) - RS2(5) - xxx
    /// Adds the contents of registers RS1 and RS2 and stores the result in register RDE.
    ADD = 0x20,

    /// OP(7) - RDE(5) - RS1(5) - RS2(5) - xxx
    /// Subtracts the contents of registers RS1 and RS2 and stores the result in register RDE.
    SUB = 0x21,

    /// OP(7) - RDE(5) - RS1(5) - RS2(5) - xxx
    /// ANDs the content of register RS1 and RS2, storing the result to register RDE.
    AND = 0x24,

    /// OP(7) - RDE(5) - RS1(5) - RS2(5) - xxx
    /// ORs the content of register RS1 and RS2, storing the result to register RDE.
    ORR = 0x25,

    /// OP(7) - RDE(5) - IMM(20)
    /// ORs the content of register RDE with the 20-bit immediate value.
    ORI = 0x26,

    /// OP(7) - RDE(5) - RS1(5) - RS2(5) - xxx
    /// XORs the content of register RS1 and RS2, storing the result to register RDE.
    XOR = 0x27,

    /// OP(7) - xxx
    /// Used to return from a branch to the previous position. Reads the last value from the
    /// "stack" and sets the program counter to it.
    RTRN = 0x3E,

    /// OP(7) - xxx
    /// Used to return from a branch to the previous position. Pops the last value from the
    /// "stack" and sets the program counter to it, freeing (setting to zero) the address where the
    /// value was stored.
    RTRN_POP = 0x3D,

    /// OP(7) - xxx
    /// Makes the core jump to its reset vector, reading the value stored inside and sets the
    /// program counter to it.
    RSET_SOFT = 0x40,

    /// OP(7) - xxx
    /// Makes the core jump to its reset vector, reading the value stored inside and sets the
    /// program counter to it. Resets all registers.
    RSET_HARD = 0x41,

    /// OP(7) - xxx
    HALT = 0x4F,

    /// OP(7) - core_index(5) - type(5)
    /// Sends an interrupt to the core specified by core_index. The type of interrupt is determined
    /// by the type specifier.
    IRPT_SEND = 0x50,
}
