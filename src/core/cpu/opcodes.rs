#[derive(PartialEq, Eq, Debug)]
#[allow(clippy::upper_case_acronyms)]
pub enum OpCode {
    // Loading OpCodes
    LDIM = 0x10,

    // Storing OpCodes
    STIM = 0x20,
    STRE = 0x21,

    // Jumping OpCodes
    JMIM = 0x30,
    JMRE = 0x31,
    JMRL = 0x32,
    JMEQ = 0x33,
    JMNE = 0x34,

    // Branching OpCodes
    BRIM = 0x35,
    BRRE = 0x36,
    BRRL = 0x37,
    BREQ = 0x38,
    BRNE = 0x39,

    // Arithmetic OpCodes
    AADD = 0x40,
    ASUB = 0x41,
    AMUL = 0x42,

    // Logic OpCodes
    LAND = 0x50,
    LORI = 0x51,
    LORR = 0x52,
    LXOR = 0x53,
    LROR = 0x54,
    LROL = 0x55,

    // System OpCodes
    HALT = 0x70,
    PUSH = 0x71,
    PULL = 0x72,
    IRPT = 0x73,
    SRES = 0x74,
    HRES = 0x75,
}