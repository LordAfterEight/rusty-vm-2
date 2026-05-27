#[derive(PartialEq, Eq, Debug)]
pub enum OpCode {
    /// Load Immediate
    ///
    /// Execution steps:
    /// 1. Gets a register index (1B)
    /// 2. Gets value (1B) to store in the GPR the index points to
    LDI = 0xA0,

    /// Store Immediate
    ///
    /// Execution steps:
    /// 1. Gets the register index to get the value from a GPR (1B)
    /// 2. Gets byte 1 of the target address (bits 0–7)
    /// 3. Gets byte 2 of the target address (bits 8–15)
    /// 4. Gets byte 3 of the target address (bits 16–23)
    /// 5. Gets byte 4 of the target address (bits 24–31)
    /// 6. Stores the value from the GPR to the assembled address
    STI = 0xA1,

    /// Jump (Absolute)
    ///
    /// Execution steps:
    /// 1. Gets byte 1 of the target address (bits 0–7)
    /// 2. Gets byte 2 of the target address (bits 8–15)
    /// 3. Gets byte 3 of the target address (bits 16–23)
    /// 4. Gets byte 4 of the target address (bits 24–31)
    /// 5. Sets IPR to the assembled address
    JMP = 0xB0,

    /// Jump (Register)
    ///
    /// Execution steps:
    /// 1. Gets the index of the GPR to pull the address from
    /// 2. Sets IPR to that value
    JMR = 0xB1,

    /// Branch (Absolute)
    ///
    /// Works the same as JMP, with the slight difference that
    /// the IPR is pushed to the stack to allow returning
    BRA = 0xB2,

    /// Branch (Register)
    ///
    /// Works the same as JMR, with the slight difference that
    /// the IPR is pushed to the stack to allow returning
    BRR = 0xB3,

    /// Return
    ///
    /// Execution steps:
    /// 1. Decreases the SPR by 1
    /// 2. Load value from the address the SPR points to into the IPR
    RTR = 0xB4,

    /// Addition (Register)
    ///
    /// Execution steps:
    /// 1. Gets index for the first GPR
    /// 2. Gets the index for the second GPR
    /// 3. Gets target GPR index
    /// 4. Adds the GPRs together
    /// 5. Stores result into the target GPR
    ADD = 0xC0,

    /// Subtraction (Register)
    ///
    /// Execution steps:
    /// 1. Gets index for the first GPR
    /// 2. Gets the index for the second GPR
    /// 3. Gets target GPR index
    /// 4. Subtracts GPR 2 from GPR 1
    /// 5. Stores result into the target GPR
    SUB = 0xC1,

    POW = 0xC2,

    /// AND (Register)
    ///
    /// Execution steps:
    /// 1. Gets index for the first GPR
    /// 2. Gets the index for the second GPR
    /// 3. Gets target GPR index
    /// 4. Performs a bitwise AND on the GPRs (GPR1 & GPR2)
    /// 5. Stores result into the target GPR
    AND = 0xC3,

    ORI = 0xC4,

    /// OR (Register)
    ///
    /// Execution steps:
    /// 1. Gets index for the first GPR
    /// 2. Gets the index for the second GPR
    /// 3. Gets target GPR index
    /// 4. Performs a bitwise OR on the GPRs (GPR1 | GPR2)
    /// 5. Stores the result into the target GPR
    ORR = 0xC5,

    /// XOR (Register)
    ///
    /// Execution steps:
    /// 1. Gets index for the first GPR
    /// 2. Gets the index for the second GPR
    /// 3. Gets target GPR index
    /// 4. Performs a bitwise XOR on the GPRs (GPR1 ^ GPR2)
    /// 5. Stores the result into the target GPR
    XOR = 0xC6,

    /// Push
    ///
    /// Execution steps:
    /// 1. Gets the index of the GPR to push
    /// 2. Pushes the GPR to the stack
    PSH = 0xD0,

    /// Pop
    ///
    /// Execution steps:
    /// 1. Gets the index of the GPR to pop to
    /// 2. Pops a value from the stack into the GPR
    POP = 0xD1,

    /// Soft Reset
    ///
    /// Execution steps:
    /// 1. Sets the IPR to the reset vector
    /// 2. Fetches byte 1 of the new address (bits 0–7)
    /// 3. Fetches byte 2 of the new address (bits 8–15)
    /// 4. Fetches byte 3 of the new address (bits 16–23)
    /// 5. Fetches byte 4 of the new address (bits 24–31)
    /// 6. Jumps to the assembled address
    ///
    /// Note: Clobbers GPRs 0 and 1
    SRE = 0xF0,

    /// Hard Reset
    ///
    /// Execution steps:
    /// 1. Sets the IPR to the reset vector
    /// 2. Fetches byte 1 of the new address (bits 0–7)
    /// 3. Fetches byte 2 of the new address (bits 8–15)
    /// 4. Fetches byte 3 of the new address (bits 16–23)
    /// 5. Fetches byte 4 of the new address (bits 24–31)
    /// 6. Jumps to the assembled address
    /// 7. Clears all GPRs
    HRE = 0xF1,

    /// Halt
    ///
    /// Execution steps:
    /// 1. Stop execution until an interrupt is received
    HLT = 0xF2,

    /// No Operation
    ///
    /// Execution steps:
    /// 1. Do nothing
    NOP = 0xF3,

    /// Interrupt Send
    ///
    /// Execution steps:
    /// 1. Gets the interrupt number to send
    /// 2. Gets the core index to send the interrupt to
    /// 3. Sends the interrupt to the specified core
    IRS = 0xF4,
}