#[repr(u32)]
#[derive(Display, num_enum::TryFromPrimitive, Debug)]
pub enum OpCode {
    NOOP = 0x00,
    LOAD = 0x01,
    STOR = 0x02,
    JUMP = 0x70,
    JUIN = 0x71,
    JUIE = 0x72,
    BRAN = 0x73,
    BRIN = 0x74,
    BRIE = 0x75,
    ADD  = 0x20,
    SUB  = 0x21,
    MUL  = 0x22,
    DIV  = 0x23,
}
