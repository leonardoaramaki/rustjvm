use std::fmt;

use crate::utils;

use super::{FieldInfo, MethodInfo};

pub const MAGIC: u32 = 0xCAFEBABE;

/// A struct representing a parsed class file.
///
/// See [class Format specs](https://docs.oracle.com/javase/specs/jvms/se7/html/jvms-4.html).
#[derive(Default, Debug, Clone)]
pub struct Class {
    pub constant_pool_count: u16,
    pub constant_pool: Vec<Constant>,
    pub access_flags: u16,
    pub this_class: u16,
    pub super_class: u16,
    pub interface_count: u16,
    pub fields: Vec<FieldInfo>,
    pub methods: Vec<MethodInfo>,
}

impl Class {
    pub fn new() -> Self {
        Self {
            constant_pool_count: 0,
            constant_pool: Vec::new(),
            access_flags: 0,
            this_class: 0,
            super_class: 0,
            interface_count: 0,
            fields: Vec::new(),
            methods: Vec::new(),
        }
    }

    /// Resolve a constant at this class constant pool.
    pub fn get_constant(&self, index: usize) -> Option<&Constant> {
        if index < 1 || index >= self.constant_pool_count as usize {
            return None;
        }
        let constant = &self.constant_pool[index];
        match constant.tag {
            // CONSTANT_Utf8
            1 => Some(constant),
            // CONSTANT_Int | // CONSTANT_Float
            3 | 4 => Some(constant),
            // CONSTANT_Long | Constant_Double
            5 | 6 => Some(constant),
            // CONSTANT_Class
            7 => {
                let name_index = utils::slice_as_u16(&constant.bytes, 0);
                self.get_constant(name_index as usize)
            },
            // CONSTANT_String
            8 => {
                let name_index = utils::slice_as_u16(&constant.bytes, 0);
                self.get_constant(name_index as usize)
            },
            // CONSTANT_Fieldref | CONSTANT_Methodref | CONSTANT_InterfaceMethodref | CONSTANT_NameAndType
            9 | 10 | 11 | 12 => Some(constant),
            _ => return None,
        }
    }

    pub fn find_method_with_name_and_descriptor(&self, name: &str, descriptor: &str) -> Option<&MethodInfo> {
        for method in self.methods.iter() {
            let const_name = self.get_constant(method.name_index as usize).unwrap();
            let const_descriptor = self.get_constant(method.descriptor_index as usize).unwrap();
            let method_name = format!("{}", const_name);
            let method_descriptor = format!("{}", const_descriptor);
            if method_name == name && method_descriptor == descriptor {
                return Some(method);
            }
        }
        None
    }

    pub fn find_field_with_name_and_descriptor(&self, name: &str, descriptor: &str) -> Option<&FieldInfo> {
        for i in 0..self.fields.len() {
            let field = self.fields.get(i).unwrap();
            let const_name = self.get_constant(field.name_index as usize).unwrap();
            let const_descriptor = self.get_constant(field.descriptor_index as usize).unwrap();
            let field_name = format!("{}", const_name);
            let field_descriptor = format!("{}", const_descriptor);
            if field_name == name && field_descriptor == descriptor {
                return self.fields.get(i);
            }
        }
        None
    }

    pub fn name(&self) -> String {
        let c = self.get_constant(self.this_class as usize).unwrap().as_string();
        c.to_string()
    }
}

#[derive(Debug, Clone)]
/// This struct represents a constant in the constant pool.
pub struct Constant {
    pub tag: u8,
    bytes: Vec<u8>,
}

impl fmt::Display for Constant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.tag == 1 {
            return write!(f, "{}", std::str::from_utf8(self.bytes.as_slice()).unwrap());
        }
        write!(f, "()")
    }
}

impl Constant {
    pub fn new(tag: u8, bytes: &[u8]) -> Self {
        Self {
            tag,
            bytes: bytes.to_vec(),
        }
    }

    pub fn as_string(&self) -> &str {
        std::str::from_utf8(self.bytes.as_slice()).unwrap()
    }

    pub fn as_int(&self) -> i32 {
        utils::slice_as_i32(&self.bytes, 0)
    }

    pub fn as_float(&self) -> f32 {
        utils::slice_as_f32(&self.bytes, 0)
    }

    pub fn as_long(&self) -> (i32, i32) {
        let msb: i32 = i32::from_be_bytes([ self.bytes[0], self.bytes[1], self.bytes[2], self.bytes[3], ]);
        let lsb: i32 = i32::from_be_bytes([ self.bytes[4], self.bytes[5], self.bytes[6], self.bytes[7], ]);
        (msb, lsb)
    }

    pub fn field_or_method_to_name_and_type(&self) -> (usize, usize) {
        let valid_tags = 9..11;
        assert!(valid_tags.contains(&self.tag));
        let class_index = utils::slice_as_u16(&self.bytes, 0);
        let name_and_type_index = utils::slice_as_u16(&self.bytes, 2);
        (class_index as usize, name_and_type_index as usize)
    }

    pub fn name_and_type_to_name_and_descriptor(&self) -> (usize, usize) {
        assert!(self.tag == 12);
        let name_index = utils::slice_as_u16(&self.bytes, 0);
        let descriptor_index = utils::slice_as_u16(&self.bytes, 2);
        (name_index as usize, descriptor_index as usize)
    }
}
