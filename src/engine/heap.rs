use crate::types::{Class, Object, object::make_null};

#[derive(Debug)]
pub struct Heap {
    objects: Vec<Object>,
}

impl Heap {
    pub fn new() -> Box<Self> {
        let mut objects = Vec::new();
        objects.push(make_null());
        Box::from(Self {
            objects,
        })
    }

    pub fn allocate_object(&mut self, class: &Class) -> i32 {
        let classname = class.get_constant(class.this_class as usize)
            .expect("could not resolve class name").as_string();
        let objectref = self.objects.len();
        self.objects.push(Object::new(classname.to_string(), class));
        return objectref as i32;
    }

    pub fn allocate_array(&mut self, atype: u8, count: usize) -> i32 {
        let arrayref = self.objects.len() as i32;
        let typename: &str;
        // Match the array type (https://docs.oracle.com/javase/specs/jvms/se7/html/jvms-6.html#jvms-6.5.newarray)
        match atype {
            4 => typename = "[Z",
            5 => typename = "[C",
            6 => typename = "[Z",
            8 => typename = "[B",
            9 => typename = "[S",
            10 => typename = "[I",
            _ => panic!("failed to allocate array for a double precision of invalid type"),
        }
        self.objects.push(Object::new_array(typename.to_string(), count));
        arrayref
    }

    /// Get the {Object} referenced by {objectref}.
    /// 
    /// # Panics
    /// This call *should* be unlikely to fail but it will do if {objectref} points to an invalid
    /// index inside the heap, in which case it will panic.
    pub fn get_object(&mut self, objectref: i32) -> &mut Object {
        self.objects.get_mut(objectref as usize).unwrap()
    }
}