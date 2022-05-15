// Sadly(?) I can't use an enum for this, because the list has to be exhaustive
// and I store the code as pure u8
pub const OP_CONSTANT: u8 = 0x00;
pub const OP_RETURN: u8 = 0x01;

/*#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum OpCode {
    Constant,
    Return,
}*/
