use std::{collections::HashMap, fmt::{format, Debug}};

use super::Class;

pub union Value {
    pub i: i32,
    pub s: i16,
    pub c: u16,
    pub b: i8,
}

impl Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c: u16;
        unsafe { c = self.c }
        write!(f, "{}", c)
    }
}

impl Clone for Value {
    fn clone(&self) -> Self {
        *self
    }
}

impl Copy for Value {
}


pub fn make_null() -> Object { 
    Object{
        is_array: false,
        array: Vec::new(),
        fields: HashMap::from([]),
        typename: "java/lang/Object".to_string(),
    }
}

#[derive(Debug)]
pub struct Object {
    pub typename: String,
    pub fields: HashMap<String, Field>,
    pub is_array: bool,
    array: Vec<Value>,
}

#[derive(Debug)]
pub struct Field {
    pub id: String, // in the form of name:descritor (eg: distance:I)
    pub value: i32, // the value bound to this field which could be an immediate value or a reference
}

impl Object {
    pub fn new_array(typename: String, count: usize) -> Self {
        let mut arr: Vec<Value> = Vec::with_capacity(count);
        for _ in 0..count {
            arr.push(Value { i: 0 });
        }
        Self { 
            typename, 
            fields: HashMap::new(),
            is_array: true,
            array: arr,
        }
    }

    pub fn new(typename: String, class: &Class) -> Self {
        let mut fields: HashMap<String, Field> = HashMap::new();
        for (_, field) in class.fields.iter().enumerate() {
            let name = class.get_constant(field.name_index as usize)
                .expect("could not find field name").as_string();
            let descriptor = class.get_constant(field.descriptor_index as usize)
                .expect("could not find field descriptor").as_string();
            let value = 0;
            if !field.is_static() {
                let id = format(format_args!("{}:{}", name, descriptor));
                fields.insert(id.clone(), Field { id, value });
            }
        }
        let is_array = false;
        let array = vec![];
        Self { typename, fields, is_array, array }
    }

    pub fn find_field_by_name_and_descriptor(&mut self, name: &str, descriptor: &str) -> Option<&mut Field> {
        let id = format!("{name}:{descriptor}");
        self.fields.get_mut(&id)
    }

    pub fn set_array_value(&mut self, index: usize, value: Value) {
        self.array[index] = value;
    }

    pub fn get_array_value(&self, index: usize) -> Value {
        self.array[index]
    }

    pub fn get_array_length(&self) -> usize {
        self.array.len()
    }
}

impl Field {
    pub fn set_value(&mut self, value: i32) {
        self.value = value;
    }
}
