use std::cell::Cell;
use std::fs::File;
use std::io::Read;

use crate::types::attributes::AttributeInfo;
use crate::types::class::{Class, Constant, MAGIC};
use crate::types::field_info::FieldInfo;
use crate::types::method_info::MethodInfo;
use crate::utils;

pub fn load_class_from(path: &str) -> Class {
    let result = File::open(path);
    if result.is_err() && !path.starts_with(".") {
        return load_class_from(format!("./{}", path.split("/").last().unwrap()).as_str());
    }
    let mut opened: File;
    let typename = path.split('.').next().unwrap();
    match result {
        Err(_) => panic!("{} could not be found on classpath",  typename),
        Ok(file) => opened = file,
    }
    let mut buffer = Vec::new();
    opened.read_to_end(&mut buffer).unwrap();
    let bytes = buffer.as_slice();
    return parse_class_file(bytes)
}

pub fn load_class_with_classpath(classpath: &str, path: &str) -> Class {
    return load_class_from(format!("{classpath}/{path}").as_str());
}

/// Returns a {Class} by parsing a given byte array which got read from a file
/// in the class file format.
///
/// # Panics
///
/// Panics if magic number is not 0xcafebabe.
pub fn parse_class_file(bytes: &[u8]) -> Class {
    let mut idx = 0;
    let mut class_file = Class::new();
    let magic: u32 = utils::slice_as_u32(bytes, idx);

    if magic != MAGIC {
        panic!("not a Java class file")
    }
    idx = 8; // skip: magic, minor and major versions
    class_file.constant_pool_count = utils::slice_as_u16(bytes, idx);
    idx += 2;

    // Unused constant at index 0.
    class_file.constant_pool = Vec::with_capacity(class_file.constant_pool_count as usize + 1);
    for _ in 1..class_file.constant_pool_count {
        class_file.constant_pool.push(Constant::new(0, &[]));
    }

    let mut skip: bool = false; // should skip one item in the pool if last one was either a long or double
    for i in 1..class_file.constant_pool_count {
        if skip {
            skip = false;
            continue;
        }
        let tag = bytes[idx];
        match tag {
            // CONSTANT_Utf8
            1 => {
                idx += 1; // skip tag
                let length: u16 = u16::from_be_bytes([ bytes[idx], bytes[idx + 1] ]);
                idx += 2; // skip length
                let data = &bytes[idx..idx + length as usize];
                let constant = Constant::new(tag, data);
                class_file.constant_pool.insert(i as usize, constant);
                idx += length as usize;
            }
            // CONSTANT_Integer | CONSTANT_Float
            3 | 4 => {
                idx += 1;
                let data = &bytes[idx..idx + 4];
                let constant = Constant::new(tag, data);
                class_file.constant_pool.insert(i as usize, constant);
                idx += 4;
            }
            // CONSTANT_Long | CONSTANT_Double
            5 | 6 => {
                idx += 1;
                let data = &bytes[idx..idx + 8];
                let constant = Constant::new(tag, data);
                class_file.constant_pool.insert(i as usize, constant);
                idx += 8;
                skip = true;
            }
            // CONSTANT_Class
            7 => {
                idx += 1;
                let data = &bytes[idx..idx + 2];
                let constant = Constant::new(tag, data);
                class_file.constant_pool.insert(i as usize, constant);
                idx += 2;
            }
            // CONSTANT_String
            8 => {
                idx += 1;
                let data = &bytes[idx..idx + 2];
                let constant = Constant::new(tag, data);
                class_file.constant_pool.insert(i as usize, constant);
                idx += 2;
            }
            // CONSTANT_Fieldref | CONSTANT_Methodref | CONSTANT_InterfaceMethodref
            9 | 10 | 11 => {
                idx += 1;
                let data = &bytes[idx..idx + 4];
                let constant = Constant::new(tag, data);
                class_file.constant_pool.insert(i as usize, constant);
                idx += 4;
            }
            // CONSTANT_NameAndType
            12 => {
                idx += 1;
                let data = &bytes[idx..idx + 4];
                let constant = Constant::new(tag, data);
                class_file.constant_pool.insert(i as usize, constant);
                idx += 4;
            }
            _ => todo!("constant tag: {}", tag),
        }
    }

    class_file.access_flags = utils::slice_as_u16(bytes, idx);
    idx += 2;
    class_file.this_class = utils::slice_as_u16(bytes, idx);
    idx += 2;
    class_file.super_class = utils::slice_as_u16(bytes, idx);
    idx += 2;
    class_file.interface_count = utils::slice_as_u16(bytes, idx);
    idx += 2;
    let mut interfaces: Vec<u16> = Vec::new();
    for _ in 0..class_file.interface_count {
        let interface_idx = utils::slice_as_u16(bytes, idx);
        idx += 2;
        interfaces.push(interface_idx);
    }

    let fields_count = utils::slice_as_u16(bytes, idx);
    idx += 2;
    for _ in 0..fields_count {
        let access_flags = utils::slice_as_u16(bytes, idx);
        idx += 2;
        let name_index = utils::slice_as_u16(bytes, idx);
        idx += 2;
        let descriptor_index = utils::slice_as_u16(bytes, idx);
        idx += 2;
        let attributes_count = utils::slice_as_u16(bytes, idx);
        idx += 2;
        let attribute_info: Vec<AttributeInfo> = Vec::new();
        let mut field = FieldInfo {
            access_flags,
            name_index,
            descriptor_index,
            attributes_count,
            attribute_info,
            value: Cell::new(0),
        };

        for _ in 0..attributes_count {
            let attribute_name_index = utils::slice_as_u16(bytes, idx);
            idx += 2;
            let attribute_length = utils::slice_as_u32(bytes, idx);
            idx += 4;
            let mut info: Vec<u8> = Vec::new();
            let data = &bytes[idx..idx + attribute_length as usize];
            idx += attribute_length as usize;
            info.extend_from_slice(data);
            let attr = AttributeInfo {
                attribute_name_index,
                attribute_length,
                info,
            };
            field.attribute_info.push(attr);
        }
        class_file.fields.push(field);
    }

    let methods_count = utils::slice_as_u16(bytes, idx);
    idx += 2;
    for _ in 0..methods_count {
        let access_flags = utils::slice_as_u16(bytes, idx);
        idx += 2;
        let name_index = utils::slice_as_u16(bytes, idx);
        idx += 2;
        let descriptor_index = utils::slice_as_u16(bytes, idx);
        idx += 2;
        let attributes_count = utils::slice_as_u16(bytes, idx);
        idx += 2;
        let attribute_info: Vec<AttributeInfo> = Vec::new();
        let mut method = MethodInfo {
            access_flags,
            name_index,
            descriptor_index,
            attributes_count,
            attribute_info,
        };

        for _ in 0..attributes_count {
            let attribute_name_index = utils::slice_as_u16(bytes, idx);
            idx += 2;
            let attribute_length = utils::slice_as_u32(bytes, idx);
            idx += 4;
            let mut info: Vec<u8> = Vec::new();
            let data = &bytes[idx..idx + attribute_length as usize];
            idx += attribute_length as usize;
            info.extend_from_slice(data);
            let attr = AttributeInfo {
                attribute_name_index,
                attribute_length,
                info,
            };
            method.attribute_info.push(attr);
        }
        class_file.methods.push(method);
    }

    let attributes_count = utils::slice_as_u16(bytes, idx);
    idx += 2;
    for _ in 0..attributes_count {
        let attribute_name_index = utils::slice_as_u16(bytes, idx);
        idx += 2;
        let attribute_length = utils::slice_as_u32(bytes, idx);
        idx += 4;
        let mut info: Vec<u8> = Vec::new();
        let data = &bytes[idx..idx + attribute_length as usize];
        idx += attribute_length as usize;
        info.extend_from_slice(data);
        let _attr = AttributeInfo {
            attribute_name_index: attribute_name_index,
            attribute_length: attribute_length,
            info: info,
        };
        //TODO: add this to class file
    }

    return class_file;
}
