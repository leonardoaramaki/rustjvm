#[derive(Debug, Clone)]
pub struct AttributeInfo {
    pub attribute_name_index: u16,
    pub attribute_length: u32,
    pub info: Vec<u8>,
}

pub trait CodeAttribute {
    fn max_stack(&self) -> u16;
    fn max_locals(&self) -> u16;
    fn code_length(&self) -> u32;
    fn code(&self) -> &[u8];
}
