use super::{attributes::AttributeInfo, attributes::CodeAttribute, Class};
use crate::utils;

#[derive(Debug, Clone)]
pub struct MethodInfo {
    pub access_flags: u16,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes_count: u16,
    pub attribute_info: Vec<AttributeInfo>,
}

pub trait NativeMethod {
    fn get() -> dyn Fn()->*const();
}

impl MethodInfo {
    pub fn is_native(&self) -> bool {
        self.access_flags & 0x0100 == 0x0100
    }

    pub fn is_static(&self) -> bool {
        self.access_flags & 0x0008 == 0x0008
    }

    pub fn get_code_attribute(&self, class: &Class) -> Option<impl CodeAttribute> {
        for attr in self.attribute_info.iter() {
            let constant_name = class
                .get_constant(attr.attribute_name_index as usize)
                .expect("attribute name not found on class constant pool");
            if constant_name.as_string() == "Code" {
                struct Temp {
                    data: Vec<u8>,
                }
                impl CodeAttribute for Temp {
                    fn max_stack(&self) -> u16 {
                        utils::slice_as_u16(self.data.as_slice(), 0)
                    }

                    fn max_locals(&self) -> u16 {
                        utils::slice_as_u16(self.data.as_slice(), 2)
                    }

                    fn code_length(&self) -> u32 {
                        utils::slice_as_u32(self.data.as_slice(), 4)
                    }

                    fn code(&self) -> &[u8] {
                        &self.data.as_slice()[8..]
                    }
                }
                return Some(Temp {
                    data: attr.info.clone(),
                });
            }
        }
        None
    }
}
