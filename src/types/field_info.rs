use std::cell::Cell;

use super::AttributeInfo;

#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub access_flags: u16,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes_count: u16,
    pub attribute_info: Vec<AttributeInfo>,
    pub value: Cell<i32>,
}

impl FieldInfo {
    pub fn set_value(&self, value: i32) {
        self.value .set(value)
    }

    pub fn is_static(&self) -> bool {
        self.access_flags & (0x0008) == 0x0008
    }
}
