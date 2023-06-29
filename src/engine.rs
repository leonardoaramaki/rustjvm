mod opcodes;
mod heap;
mod classloader;
mod ops;
mod jni;

use crate::types::Class;
use crate::types::{Value, attributes::CodeAttribute, frame::Frame, Location};

use opcodes::*;
use heap::Heap;
use classloader::Classloader;
use core::str;
use std::{collections::HashMap, rc::Rc};


pub struct Runtime {
    heap: Box<Heap>,
    classloader: Box<Classloader>,
    frame_stack: Vec<Box<Frame>>,
    stringpool: HashMap<String, i32>,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            heap: Heap::new(),
            classloader: Classloader::new(),
            frame_stack: Vec::new(),
            stringpool: HashMap::new(),
        }
    }

    fn load_and_initialize(&mut self, classname: &str) -> Rc<Class> {
        let optional = self.classloader.find_loaded_class(classname);
        let class: Rc<Class>;
        let mut first_time_loaded = false;
        if optional.is_none() {
            first_time_loaded = true;
            class = self.classloader.load_class(classname);
        } else {
            class = optional.unwrap();
        }
        if first_time_loaded {
            self.add_static_code_frame(&class);
        }
        class.clone()
    }

    pub fn add_string_pool_feed_frame(&mut self, class: &Class) {
        let string_class = self.load_and_initialize("java/lang/String");
        for (i, constant) in class.constant_pool.iter().enumerate() {
            let is_string_const = constant.tag == 8;
            if is_string_const {
                let str = class.get_constant(i).unwrap().as_string();
                if self.stringpool.contains_key(str) {
                    continue;
                }
                let objectref = self.heap.allocate_object(&string_class);
                self.stringpool.insert(str.to_string(), objectref);
                // Invoke <init>:([C)V on java/lang/String class
                let result = string_class.find_method_with_name_and_descriptor("<init>", "([C)V");
                if let Some(string_ctor) = result {
                    let code_attribute = string_ctor
                        .get_code_attribute(&string_class)
                        .unwrap();
                    let max_locals = code_attribute.max_locals() as usize;
                    let max_stack = code_attribute.max_stack() as usize;
                    let location = Location::new(&string_class, string_ctor);
                    let mut frame = Frame::new(max_locals, max_stack, 0, location);
                    let arrayref = self.heap.allocate_array(5, str.len());
                    let array = self.heap.get_object(arrayref);
                    for i  in 0..str.as_bytes().len() {
                        let ch = str.as_bytes().get(i).unwrap().clone() as u16;
                        array.set_array_value(i, Value { c: ch })
                    }
                    frame.locals[0] = objectref;
                    frame.locals[1] = arrayref;
                    self.frame_stack.push(frame);
                }
            }
        }
    }

    /// Returns a String out of an Object address in heap.
    /// 
    /// # Panics
    /// A panic should happen in the following circumstances:
    /// - Not found in the heap
    /// - Not an instance of `java/lang/String`
    pub fn get_string_from_obj(&mut self, objectref: i32) -> String {
        let object = self.heap.get_object(objectref);
        if object.typename != "java/lang/String" {
            panic!("Object reference by {objectref} should be a java/lang/String instance");
        }
        let count = object.find_field_by_name_and_descriptor("count", "I")
            .unwrap().value;
        let chars_arrayref = object.find_field_by_name_and_descriptor("value", "[C")
            .unwrap().value;
        let chars_array = self.heap.get_object(chars_arrayref);
        let ch_array_len = count as usize;
        let mut v: Vec<u16> = Vec::with_capacity(ch_array_len);
        for i in 0..ch_array_len {
            let ch = unsafe { chars_array.get_array_value(i).c };
            v.push(ch);
        }
        String::from_utf16(&v).unwrap()
    }

    // pub fn add_to_stringpool(&self, )

    /// Look for the main method signature inside the class: `main([Ljava/lang/String;])V.
    pub fn check_if_can_run(&self, class: &Class) -> Result<(), &str> {
        let result = class.find_method_with_name_and_descriptor("main", "([Ljava/lang/String;)V");
        match result {
            Some(method) => return if method.is_static() { Ok(()) } else { panic!("main method should be static") },
            None => panic!("`main` method not found in class"),
        }
    }

    fn add_static_code_frame(&mut self, class: &Class) {
        let optional = class
            .find_method_with_name_and_descriptor("<clinit>", "()V");
        if optional.is_none() {
            return;
        }
        let clinit = optional.unwrap();
        let code_attribute = clinit
            .get_code_attribute(class)
            .expect("attribute type `Code` could not be found on clinit method");
        let max_locals = code_attribute.max_locals() as usize;
        let max_stack = code_attribute.max_stack() as usize;
        let location = Location::new(class, clinit);
        let frame = Frame::new(max_locals, max_stack, 0, location);
        self.frame_stack.push(frame);
    }

    pub fn entrypoint(&mut self, classname: &str, class: &Class) {
        self.load_and_initialize("java/lang/Integer");
        self.load_and_initialize("java/lang/String");
        self.load_and_initialize("java/lang/Object");
        self.interpret_next_frame();
        self.classloader.set_class_as_loaded(classname, class);
        let main = class
            .find_method_with_name_and_descriptor("main", "([Ljava/lang/String;)V")
            .expect("main method not found");
        let code_attribute = main
            .get_code_attribute(class)
            .expect("attribute type `Code` could not be found on main method");
        
        let max_locals = code_attribute.max_locals() as usize;
        let max_stack = code_attribute.max_stack() as usize;
        let location = Location::new(class, main);
        let frame = Frame::new(max_locals, max_stack, 0, location);
        self.frame_stack.push(frame);
        self.add_string_pool_feed_frame(class);
        self.add_static_code_frame(class);
        self.interpret_next_frame();
    }

    fn interpret_next_frame(&mut self) {
        use OpCode::*;
        use core::mem::size_of_val;
        loop {
            let current_frame = self.frame_stack.last_mut();
            if current_frame.is_some() {
                let current_frame = current_frame.unwrap();
                let class = &current_frame.location.declaring_type;
                let _class_name = class.name();
                let running_method = &current_frame.location.method;
                let _method_name = class.get_constant(running_method.name_index as usize).unwrap().as_string();
                let _method_descriptor = class.get_constant(running_method.descriptor_index as usize).unwrap().as_string();
                let code_attribute = running_method.get_code_attribute(class).unwrap();
                let bytes = code_attribute.code();
                let opcode = OpCode::from(bytes, current_frame.pc);
                
                // Uncomment for opcode spamming
                // if !_method_name.ends_with("<clinit>") {
                //     print!("{}#{}{}:", _class_name, _method_name, _method_descriptor);
                //     println!("\t{}: {:?}", current_frame.pc, opcode);
                // }
                current_frame.pc += 1;
                match opcode{
                    Nop =>  {},
                    IconstM1 => self.iconst_op(-1),
                    Iconst0 => self.iconst_op(0),
                    Iconst1 => self.iconst_op(1),
                    Iconst2 => self.iconst_op(2),
                    Iconst3 => self.iconst_op(3),
                    Iconst4 => self.iconst_op(4),
                    Iconst5 => self.iconst_op(5),
                    Lconst0 => self.lconst_op(0),
                    Lconst1 => self.lconst_op(1),
                    Iload { index } => {
                        current_frame.pc += size_of_val(&index);
                        self.iload_op(index as usize);
                    }
                    Lload { index } => {
                        current_frame.pc += size_of_val(&index);
                        self.lload_op(index as usize);
                    }
                    Iload0 => self.iload_op(0),
                    Iload1 => self.iload_op(1),
                    Iload2 => self.iload_op(2),
                    Iload3 => self.iload_op(3),
                    Lload0 => self.lload_op(0),
                    Lload1 => self.lload_op(1),
                    Lload2 => self.lload_op(2),
                    Lload3 => self.lload_op(3),
                    Iadd => self.iadd_op(),
                    Ladd => self.ladd_op(),
                    Isub => self.isub_op(),
                    Lsub => self.lsub_op(),
                    Imul => self.imul_op(),
                    Lmul => self.lmul_op(),
                    Idiv => self.idiv_op(),
                    Ldiv => self.ldiv_op(),
                    Irem => self.irem_op(),
                    Ineg => self.ineg_op(),
                    Ishl => self.ishl_op(),
                    Lshl => self.lshl_op(),
                    Iushr => self.iushr_op(),
                    Iand => self.iand_op(),
                    Ior => self.ior_op(),
                    I2l => self.i2l_op(),
                    L2i => self.l2i_op(),
                    I2b => self.i2b_op(),
                    I2c => self.i2c_op(),
                    Lcmp => self.lcmp_op(),
                    Iinc { index, immediate } => {
                        current_frame.pc += size_of_val(&index) + size_of_val(&immediate);
                        self.iinc_op(index, immediate);
                    }
                    IfEq { offset } => {
                        current_frame.pc += size_of_val(&offset);
                        self.if_op(offset, |v| v == 0);
                    }
                    IfNe { offset } => {
                        current_frame.pc += size_of_val(&offset);
                        self.if_op(offset, |v| v != 0);
                    }
                    IfLt { offset } => {
                        current_frame.pc += size_of_val(&offset);
                        self.if_op(offset, |v| v < 0);
                    }
                    IfLe { offset } => {
                        current_frame.pc += size_of_val(&offset);
                        self.if_op(offset, |v| v <= 0);
                    }
                    IfGt { offset } => {
                        current_frame.pc += size_of_val(&offset);
                        self.if_op(offset, |v| v > 0);
                    }
                    IfGe { offset } => {
                        current_frame.pc += size_of_val(&offset);
                        self.if_op(offset, |v| v >= 0);
                    }
                    Caload => self.caload_op(),
                    Istore { index } => {
                        current_frame.pc += size_of_val(&index);
                        self.istore_op(index as usize);
                    }
                    Lstore { index } => {
                        current_frame.pc += size_of_val(&index);
                        self.lstore_op(index as usize);
                    }
                    Istore0 => self.istore_op(0),
                    Istore1 => self.istore_op(1),
                    Istore2 => self.istore_op(2),
                    Istore3 => self.istore_op(3),
                    Lstore0 => self.lstore_op(0),
                    Lstore1 => self.lstore_op(1),
                    Lstore2 => self.lstore_op(2),
                    Lstore3 => self.lstore_op(3),
                    New { index } => {
                        current_frame.pc += size_of_val(&index);
                        self.new_op(index);
                    }
                    NewArray { atype } => {
                        current_frame.pc += size_of_val(&atype);
                        self.newarray_op(atype);
                    }
                    ArrayLength => self.arraylength_op(),
                    Dup => self.dup_op(),
                    Pop => self.pop_op(),
                    Ireturn => self.ireturn_op(),
                    Areturn => self.areturn_op(),
                    Return => {
                        self.return_op();
                    }
                    GetStatic { index } => {
                        current_frame.pc += size_of_val(&index);
                        self.getstatic_op(index);
                    }
                    Ldc { index } => {
                        current_frame.pc += size_of_val(&index);
                        self.ldc_op(index);
                    }
                    Ldc2w { index } => {
                        current_frame.pc += size_of_val(&index);
                        self.ldc2w_op(index);
                    }
                    InvokeVirtual { index } => {
                        current_frame.pc += size_of_val(&index);
                        self.invokevirtual_op(index);
                    }
                    InvokeSpecial { index } => {
                        current_frame.pc += size_of_val(&index);
                        self.invokespecial_op(index);
                    }
                    InvokeStatic { index } => {
                        current_frame.pc += size_of_val(&index);
                        self.invokestatic_op(index);
                    }
                    GetField { index } => {
                        current_frame.pc += size_of_val(&index);
                        self.getfield_op(index);
                    }
                    PutStatic { index } => {
                        current_frame.pc += size_of_val(&index);
                        self.putstatic_op(index);
                    }
                    PutField { index } => {
                        current_frame.pc += size_of_val(&index);
                        self.putfield_op(index);
                    }
                    Bipush { byte } => {
                        current_frame.pc += size_of_val(&byte);
                        self.bipush_op(byte);
                    }
                    Sipush { value } => {
                        current_frame.pc += size_of_val(&value);
                        self.sipush_op(value);
                    }
                    Aload0 => {
                        self.aload_op(0);
                    }
                    Aload1 => {
                        self.aload_op(1);
                    }
                    Aload2 => {
                        self.aload_op(2);
                    }
                    Aload3 => {
                        self.aload_op(3);
                    }
                    Iaload => self.iaload_op(),
                    Astore0 => {
                        self.astore_op(0);
                    }
                    Astore1 => {
                        self.astore_op(1);
                    }
                    Astore2 => {
                        self.astore_op(2);
                    }
                    Astore3 => {
                        self.astore_op(3);
                    }
                    Iastore => self.iastore_op(),
                    Castore => self.castore_op(),
                    IfICmpEq { offset } => {
                        current_frame.pc += size_of_val(&offset);
                        self.if_icmp_op(offset, |a, b| a == b);
                    }
                    IfICmpNe { offset } => {
                        current_frame.pc += size_of_val(&offset);
                        self.if_icmp_op(offset, |a, b| a != b);
                    }
                    IfICmpGe { offset } => {
                        current_frame.pc += size_of_val(&offset);
                        self.if_icmp_op(offset, |a, b| a >= b);
                    }
                    IfICmpGt { offset } => {
                        current_frame.pc += size_of_val(&offset);
                        self.if_icmp_op(offset, |a, b| a > b);
                    }
                    IfICmpLe { offset } => {
                        current_frame.pc += size_of_val(&offset);
                        self.if_icmp_op(offset, |a, b| a <= b);
                    }
                    IfICmpLt { offset } => {
                        current_frame.pc += size_of_val(&offset);
                        self.if_icmp_op(offset, |a, b| a < b);
                    }
                    Goto { offset } => {
                        current_frame.pc += size_of_val(&offset);
                        self.goto_op(offset);
                    }
                    IfNull { offset } => {
                        current_frame.pc += size_of_val(&offset);
                        self.ifnull_op(offset);
                    }
                    IfNonNull { offset } => {
                        current_frame.pc += size_of_val(&offset);
                        self.ifnonnull_op(offset);
                    }
                }
            } else {
                break;
            }
        }
    }
}
