use std::rc::Rc;

use crate::types::attributes::CodeAttribute;
use crate::types::frame::Frame;
use crate::types::{Field, Value, Location, Class};
use super::Runtime;
use super::jni::*;

impl Runtime {

    #[inline(always)]
    pub fn ldc_op(&mut self, index: u8) {
        let frame_stack: &mut Vec<Box<Frame>> = &mut self.frame_stack;
        if let Some(current_frame) = frame_stack.last_mut() {
            let class = &current_frame.location.declaring_type;
            let constant = class.get_constant(index as usize).unwrap();
            match constant.tag {
                // CONSTANT_Utf8
                1 => {
                    let str = constant.as_string();
                    let stringref = self.stringpool[str];
                    current_frame.operands.push(stringref);
                }
                // CONSTANT_Integer
                3 => current_frame.operands.push(constant.as_int()),
                // CONSTANT_Float
                4 => current_frame.operands.push(constant.as_float().to_bits() as i32),
                _ => panic!("ldc does not support constant tag: {}", constant.tag)
            }
        }
    }

    #[inline(always)]
    pub fn ldc2w_op(&mut self, index: u16) {
        let frame_stack: &mut Vec<Box<Frame>> = &mut self.frame_stack;
        if let Some(current_frame) = frame_stack.last_mut() {
            let class = &current_frame.location.declaring_type;
            let constant = class.get_constant(index as usize).unwrap();
            match constant.tag {
                // CONSTANT_Long | CONSTANT_Double
                5 | 6 => {
                    let (msb, lsb) = constant.as_long();
                    current_frame.operands.push(msb);
                    current_frame.operands.push(lsb);
                }
                _ => panic!("ldc2_w does not support constant tag: {}", constant.tag)
            }
        }
    }

    #[inline(always)]
    pub fn iconst_op(&mut self, value: i8) {
        let frame_stack: &mut Vec<Box<Frame>> = &mut self.frame_stack;
        if let Some(current_frame) = frame_stack.last_mut() {
            current_frame.operands.push(value as i32);
        }
    }

    #[inline(always)]
    pub fn lconst_op(&mut self, value: i8) {
        let frame_stack: &mut Vec<Box<Frame>> = &mut self.frame_stack;
        if let Some(current_frame) = frame_stack.last_mut() {
            current_frame.operands.push(value as i32);
            current_frame.operands.push(0 as i32);
        }
    }

    #[inline(always)]
    pub fn iadd_op(&mut self) {
        let frame_stack: &mut Vec<Box<Frame>> = &mut self.frame_stack;
        if let Some(current_frame) = frame_stack.last_mut() {
            let value2 = current_frame.operands.pop().unwrap();
            let value1 = current_frame.operands.pop().unwrap();
            current_frame.operands.push(value1 + value2);
        }
    }

    #[inline(always)]
    pub fn ladd_op(&mut self) {
        let frame_stack: &mut Vec<Box<Frame>> = &mut self.frame_stack;
        if let Some(current_frame) = frame_stack.last_mut() {
            let value2_low = current_frame.operands.pop().unwrap();
            let value2_high = current_frame.operands.pop().unwrap();
            let value1_low = current_frame.operands.pop().unwrap();
            let value1_high = current_frame.operands.pop().unwrap();
            
            let (sum_low, carry) = value1_low.carrying_add(value2_low, false);
            let (sum_high, _) = value1_high.carrying_add(value2_high, carry);
            current_frame.operands.push(sum_high);
            current_frame.operands.push(sum_low);
        }
    }

    #[inline(always)]
    pub fn isub_op(&mut self) {
        let frame_stack: &mut Vec<Box<Frame>> = &mut self.frame_stack;
        if let Some(current_frame) = frame_stack.last_mut() {
            let value2 = current_frame.operands.pop().unwrap();
            let value1 = current_frame.operands.pop().unwrap();
            current_frame.operands.push(value1 - value2);
        }
    }

    #[inline(always)]
    pub fn lsub_op(&mut self) {
        let frame_stack: &mut Vec<Box<Frame>> = &mut self.frame_stack;
        if let Some(current_frame) = frame_stack.last_mut() {
            let value2_low = current_frame.operands.pop().unwrap();
            let value2_high = current_frame.operands.pop().unwrap();
            let value1_low = current_frame.operands.pop().unwrap();
            let value1_high = current_frame.operands.pop().unwrap();

            let (diff_low, borrow) = value1_low.borrowing_sub(value2_low, false);
            let (diff_high, _) = value1_high.carrying_add(value2_high, borrow);
            current_frame.operands.push(diff_high);
            current_frame.operands.push(diff_low);
        }
    }

    #[inline(always)]
    pub fn imul_op(&mut self) {
        let frame_stack: &mut Vec<Box<Frame>> = &mut self.frame_stack;
        if let Some(current_frame) = frame_stack.last_mut() {
            let value2 = current_frame.operands.pop().unwrap();
            let value1 = current_frame.operands.pop().unwrap();
            current_frame.operands.push(value1 * value2);
        }
    }

    #[inline(always)]
    pub fn lmul_op(&mut self) {
        let frame_stack: &mut Vec<Box<Frame>> = &mut self.frame_stack;
        if let Some(current_frame) = frame_stack.last_mut() {
            #[cfg(all(target_pointer_width = "64"))] {
                let value2_low = current_frame.operands.pop().unwrap();
                let value2_high = current_frame.operands.pop().unwrap();
                let value1_low = current_frame.operands.pop().unwrap();
                let value1_high = current_frame.operands.pop().unwrap();
                
                let value2 = (value2_high as i64) << 32 | value2_low as i64;
                let value1 = (value1_high as i64) << 32 | value1_low as i64;
                let result: i64 = value1 * value2;

                current_frame.operands.push((result >> 32) as i32);
                current_frame.operands.push(result as i32);
            }
        }
    }

    #[inline(always)]
    pub fn idiv_op(&mut self) {
        let frame_stack: &mut Vec<Box<Frame>> = &mut self.frame_stack;
        if let Some(current_frame) = frame_stack.last_mut() {
            let value2 = current_frame.operands.pop().unwrap();
            let value1 = current_frame.operands.pop().unwrap();
            current_frame.operands.push(value1 / value2);
        }
    }

    #[inline(always)]
    pub fn ldiv_op(&mut self) {
        let frame_stack: &mut Vec<Box<Frame>> = &mut self.frame_stack;
        if let Some(current_frame) = frame_stack.last_mut() {
            let value2_lsb = current_frame.operands.pop().unwrap();
            let value2_msb = current_frame.operands.pop().unwrap();
            let value1_lsb = current_frame.operands.pop().unwrap();
            let value1_msb = current_frame.operands.pop().unwrap();
            
            #[cfg(all(target_pointer_width = "64"))] 
            { 
                let value1: i64 = (value1_msb as i64) << 32 | value1_lsb as i64;
                let value2: i64 = (value2_msb as i64) << 32 | value2_lsb as i64;
                let quotient = value1 / value2;
                current_frame.operands.push((quotient >> 32) as i32);
                current_frame.operands.push((quotient & 0xFFFFFFFF) as i32);
            }

            #[cfg(all(target_pointer_width = "32"))] 
            { 
                panic!("32-bit arch not supported");
            }
        }
    }

    #[inline(always)]
    pub fn irem_op(&mut self) {
        let frame_stack: &mut Vec<Box<Frame>> = &mut self.frame_stack;
        if let Some(current_frame) = frame_stack.last_mut() {
            let value2 = current_frame.operands.pop().unwrap();
            let value1 = current_frame.operands.pop().unwrap();
            current_frame.operands.push(value1 - (value1 / value2) * value2);
        }
    }

    #[inline(always)]
    pub fn ineg_op(&mut self) {
        let frame_stack: &mut Vec<Box<Frame>> = &mut self.frame_stack;
        if let Some(current_frame) = frame_stack.last_mut() {
            let value = current_frame.operands.pop().unwrap();
            current_frame.operands.push(value * -1);
        }
    }

    #[inline(always)]
    pub fn iushr_op(&mut self) {
        let frame_stack: &mut Vec<Box<Frame>> = &mut self.frame_stack;
        if let Some(current_frame) = frame_stack.last_mut() {
            let value2 = current_frame.operands.pop().unwrap();
            let value1 = current_frame.operands.pop().unwrap();
            let s = value2 & 0b11111;
            current_frame.operands.push(value1 >> s);
        }
    }

    #[inline(always)]
    pub fn ishl_op(&mut self) {
        let frame_stack: &mut Vec<Box<Frame>> = &mut self.frame_stack;
        if let Some(current_frame) = frame_stack.last_mut() {
            let value2 = current_frame.operands.pop().unwrap();
            let value1 = current_frame.operands.pop().unwrap();
            let s = value2 & 0b11111;
            current_frame.operands.push(value1 << s);
        }
    }

    #[inline(always)]
    pub fn lshl_op(&mut self) {
        let frame_stack: &mut Vec<Box<Frame>> = &mut self.frame_stack;
        if let Some(current_frame) = frame_stack.last_mut() {
            let value2 = current_frame.operands.pop().unwrap();
            let value1_low = current_frame.operands.pop().unwrap();
            let value1_high = current_frame.operands.pop().unwrap();
            let s: i32 = value2  & 0b11111;
            current_frame.operands.push(value1_high << s);
            current_frame.operands.push(value1_low << s);
        }
    }

    #[inline(always)]
    pub fn iand_op(&mut self) {
        let frame_stack: &mut Vec<Box<Frame>> = &mut self.frame_stack;
        if let Some(current_frame) = frame_stack.last_mut() {
            let value2 = current_frame.operands.pop().unwrap();
            let value1 = current_frame.operands.pop().unwrap();
            current_frame.operands.push(value1 & value2);
        }
    }

    #[inline(always)]
    pub fn ior_op(&mut self) {
        let frame_stack: &mut Vec<Box<Frame>> = &mut self.frame_stack;
        if let Some(current_frame) = frame_stack.last_mut() {
            let value2 = current_frame.operands.pop().unwrap();
            let value1 = current_frame.operands.pop().unwrap();
            current_frame.operands.push(value1 & value2);
        }
    }

    #[inline(always)]
    pub fn i2l_op(&mut self) {
        let frame_stack: &mut Vec<Box<Frame>> = &mut self.frame_stack;
        if let Some(current_frame) = frame_stack.last_mut() {
            let low = current_frame.operands.pop().unwrap();
            let high: i32 = if low < 0 { core::i32::MIN } else { core::i32::MAX };
            current_frame.operands.push(high);
            current_frame.operands.push(low); 
        }
    }

    #[inline(always)]
    pub fn l2i_op(&mut self) {
        let frame_stack: &mut Vec<Box<Frame>> = &mut self.frame_stack;
        if let Some(current_frame) = frame_stack.last_mut() {
            let low = current_frame.operands.pop().unwrap();
            let _ = current_frame.operands.pop().unwrap();
            current_frame.operands.push(low); 
        }
    }

    #[inline(always)]
    pub fn i2b_op(&mut self) {
        let frame_stack: &mut Vec<Box<Frame>> = &mut self.frame_stack;
        if let Some(current_frame) = frame_stack.last_mut() {
            let value = current_frame.operands.pop().unwrap() as i8;
            current_frame.operands.push(value as i32);
        }
    }

    #[inline(always)]
    pub fn i2c_op(&mut self) {
        let frame_stack: &mut Vec<Box<Frame>> = &mut self.frame_stack;
        if let Some(current_frame) = frame_stack.last_mut() {
            let value = current_frame.operands.pop().unwrap() as u16;
            current_frame.operands.push(value as i32);
        }
    }

    #[inline(always)]
    pub fn lcmp_op(&mut self) {
        let frame_stack: &mut Vec<Box<Frame>> = &mut self.frame_stack;
        if let Some(current_frame) = frame_stack.last_mut() {
            let value2_low = current_frame.operands.pop().unwrap();
            let value2_high = current_frame.operands.pop().unwrap();
            let value1_low = current_frame.operands.pop().unwrap();
            let value1_high = current_frame.operands.pop().unwrap();
            #[cfg(any(target_pointer_width = "64"))]
            {
                let value2 = (value2_high as i64) << 32 | value2_low as i64;
                let value1 = (value1_high as i64) << 32 | value1_low as i64;
                let result = match value1 {
                    _x if value1 > value2 => 1,
                    _x if value1 == value2 => 0,
                    _x if value1 < value2 => -1,
                    _ => unreachable!(),
                };
                current_frame.operands.push(result);
            }
        }
    }

    #[inline(always)]
    pub fn iinc_op(&mut self, index: u8, immediate: i8) {
        let frame_stack: &mut Vec<Box<Frame>> = &mut self.frame_stack;
        if let Some(current_frame) = frame_stack.last_mut() {
            current_frame.locals[index as usize] += immediate as i32;
        }
    }

    #[inline(always)]
    pub fn istore_op(&mut self, index: usize) {
        let frame_stack: &mut Vec<Box<Frame>> = &mut self.frame_stack;
        if let Some(current_frame) = frame_stack.last_mut() {
            let value = current_frame.operands.pop().unwrap();
            current_frame.locals[index] = value;
        }
    }

    #[inline(always)]
    pub fn lstore_op(&mut self, index: usize) {
        let frame_stack: &mut Vec<Box<Frame>> = &mut self.frame_stack;
        if let Some(current_frame) = frame_stack.last_mut() {
            let low = current_frame.operands.pop().unwrap();
            let high = current_frame.operands.pop().unwrap();
            current_frame.locals[index] = high;
            current_frame.locals[index + 1] = low;
        }
    }

    #[inline(always)]
    pub fn iload_op(&mut self, index: usize) {
        let frame_stack: &mut Vec<Box<Frame>> = &mut self.frame_stack;
        if let Some(current_frame) = frame_stack.last_mut() {
            let value = current_frame.locals.get(index).unwrap();
            current_frame.operands.push(*value);
        }
    }

    #[inline(always)]
    pub fn lload_op(&mut self, index: usize) {
        let frame_stack: &mut Vec<Box<Frame>> = &mut self.frame_stack;
        if let Some(current_frame) = frame_stack.last_mut() {
            let msb = current_frame.locals.get(index).unwrap();
            let lsb = current_frame.locals.get(index + 1).unwrap();
            current_frame.operands.push(*msb);
            current_frame.operands.push(*lsb);
        }
    }

    #[inline(always)]
    pub fn iaload_op(&mut self) {
        let frame_stack: &mut Vec<Box<Frame>> = &mut self.frame_stack;
        if let Some(current_frame) = frame_stack.last_mut() {
            let index = current_frame.operands.pop().unwrap();
            let arrayref = current_frame.operands.pop().unwrap();
            let array = self.heap.get_object(arrayref);
            let value = array.get_array_value(index.try_into().unwrap());
            unsafe { current_frame.operands.push(value.i) };
        }
    }

    #[inline(always)]
    pub fn bipush_op(&mut self, byte: i8) {
        let frame_stack: &mut Vec<Box<Frame>> = &mut self.frame_stack;
        if let Some(current_frame) = frame_stack.last_mut() {
            current_frame.operands.push((byte as i32) << 24 >> 24);
        }
    }

    #[inline(always)]
    pub fn sipush_op(&mut self, value: i16) {
        let frame_stack: &mut Vec<Box<Frame>> = &mut self.frame_stack;
        if let Some(current_frame) = frame_stack.last_mut() {
            current_frame.operands.push(value as i32);
        }
    }

    #[inline(always)]
    pub fn new_op(&mut self, index: u16) {
        let framestack: &mut Vec<Box<Frame>> = &mut self.frame_stack;
        if let Some(current_frame) = framestack.last_mut() {
            let current_class = &current_frame.location.declaring_type;
            let class_name = current_class.get_constant(index as usize)
                .expect("expected a class name").as_string();
            let optional = self.classloader.find_loaded_class(class_name);
            let class: Rc<Class>;
            let mut first_time_loaded = false;
            if optional.is_none() {
                first_time_loaded = true;
                class = self.classloader.load_class(class_name);
            } else {
                class = optional.unwrap();
            }
            let objectref = self.heap.allocate_object(&class);
            current_frame.operands.push(objectref);
            if first_time_loaded {
                self.add_string_pool_feed_frame(&class);
                self.add_static_code_frame(&class);
            }
        }
    }

    #[inline(always)]
    pub fn dup_op(&mut self) {
        let frame_stack: &mut Vec<Box<Frame>> = &mut self.frame_stack;
        if let Some(current_frame) = frame_stack.last_mut() {
            let value = current_frame.operands.last().unwrap();
            current_frame.operands.push(*value);
        }
    }

    #[inline(always)]
    pub fn pop_op(&mut self) {
        let frame_stack: &mut Vec<Box<Frame>> = &mut self.frame_stack;
        if let Some(current_frame) = frame_stack.last_mut() {
            current_frame.operands.pop();
        }
    }

    #[inline(always)]
    pub fn invokespecial_op(&mut self, index: u16) {
        let frame_stack: &mut Vec<Box<Frame>> = &mut self.frame_stack;
        if let Some(current_frame) = frame_stack.last_mut() {
            let class = &current_frame.location.declaring_type;
            let (name_index, name_and_type_index) = class.get_constant(index as usize).unwrap()
                .field_or_method_to_name_and_type();
            let (method_name_index, method_descriptor_index) = class.get_constant(name_and_type_index)
                .unwrap().name_and_type_to_name_and_descriptor();

            let class_name = class.get_constant(name_index).unwrap().as_string();
            let method_name = class.get_constant(method_name_index).unwrap().as_string();
            let method_descriptor = class.get_constant(method_descriptor_index).unwrap().as_string();

            let optional = self.classloader.find_loaded_class(class_name);
            let class: Rc<Class>;
            let mut first_time_loaded = false;
            if optional.is_none() {
                first_time_loaded = true;
                class = self.classloader.load_class(class_name);
            } else {
                class = optional.unwrap();
            }
            let method = class.find_method_with_name_and_descriptor(method_name, method_descriptor).unwrap();
            let code_attribute = method.get_code_attribute(&class).unwrap();
            let max_locals = code_attribute.max_locals() as usize;
            let max_stack = code_attribute.max_stack() as usize;
            let location = Location::new(&class, method);
            let types = parse_method_descriptor(method_descriptor);

            let mut new_frame = Frame::new(max_locals, max_stack, 0, location);

            for i in (0..types.len()).rev() {
                new_frame.locals[i] = current_frame.operands.pop().unwrap();
            }
            self.frame_stack.push(new_frame);
            if first_time_loaded {
                self.add_static_code_frame(&class);
                self.add_string_pool_feed_frame(&class);
            }
        }
    }

    #[inline(always)]
    pub fn invokevirtual_op(&mut self, index: u16) {
        let frame_stack: &mut Vec<Box<Frame>> = &mut self.frame_stack;
        if let Some(current_frame) = frame_stack.last_mut() {
            let class = &current_frame.location.declaring_type;
            let (name_index, name_and_type_index) = class.get_constant(index as usize).unwrap()
                .field_or_method_to_name_and_type();
            let (method_name_index, method_descriptor_index) = class.get_constant(name_and_type_index)
                .unwrap().name_and_type_to_name_and_descriptor();

            let class_name = class.get_constant(name_index).unwrap().as_string();
            let method_name = class.get_constant(method_name_index).unwrap().as_string();
            let method_descriptor = class.get_constant(method_descriptor_index).unwrap().as_string();

            let optional = self.classloader.find_loaded_class(class_name);
            let class: Rc<Class>;
            let mut first_time_loaded = false;
            if optional.is_none() {
                first_time_loaded = true;
                class = self.classloader.load_class(class_name);
            } else {
                class = optional.unwrap();
            }
            let method = class.find_method_with_name_and_descriptor(method_name, method_descriptor).unwrap();
            let location = Location::new(&class, method);
            let types = parse_method_descriptor(method_descriptor);

            if let Some(code_attribute) = method.get_code_attribute(&class) {
                let max_locals = code_attribute.max_locals() as usize;
                let max_stack = code_attribute.max_stack() as usize;
    
                let mut new_frame = Frame::new(max_locals, max_stack, 0, location);
    
                let nargs = types.iter().fold(0, |acc, x| if x == "J" || x == "D" { acc + 2 } else { acc + 1 }) - 1;
                for i in (1..nargs + 1).rev() {
                    new_frame.locals[i] = current_frame.operands.pop()
                        .expect(&format!("Failed to resolve {}.{}", class_name, method_name));
                }
                let objectref = current_frame.operands.pop().unwrap();
                new_frame.locals[0] = objectref;
                if first_time_loaded {
                    self.add_static_code_frame(&class);
                    self.add_string_pool_feed_frame(&class);
                }
                self.frame_stack.push(new_frame);
                return;
            }

            if method.is_native() {
                let max_locals = types.len() - 1;
                let max_stack = 0;
                let mut new_frame = Frame::new(max_locals, max_stack, 0, location);
                for i in (0..max_locals).rev() {
                    new_frame.locals[i] = current_frame.operands.pop().unwrap();
                }
                let native_call = get_assoc_native_method(class_name, method_name, method_descriptor);
                native_call(self, &new_frame);
            }
        }
    }

    #[inline(always)]
    pub fn invokestatic_op(&mut self, index: u16) {
        let frame_stack: &mut Vec<Box<Frame>> = &mut self.frame_stack;
        if let Some(current_frame) = frame_stack.last_mut() {
            let class = &current_frame.location.declaring_type;
            let (name_index, name_and_type_index) = class.get_constant(index as usize).unwrap()
                .field_or_method_to_name_and_type();
            let (method_name_index, method_descriptor_index) = class.get_constant(name_and_type_index)
                .unwrap().name_and_type_to_name_and_descriptor();

            let class_name = class.get_constant(name_index).unwrap().as_string();
            let method_name = class.get_constant(method_name_index).unwrap().as_string();
            let method_descriptor = class.get_constant(method_descriptor_index).unwrap().as_string();

            let optional = self.classloader.find_loaded_class(class_name);
            let class: Rc<Class>;
            let mut first_time_loaded = false;
            if optional.is_none() {
                first_time_loaded = true;
                class = self.classloader.load_class(class_name);
            } else {
                class = optional.unwrap();
            }
            let method = class.find_method_with_name_and_descriptor(method_name, method_descriptor).unwrap();
            let location = Location::new(&class, method);
            let types = parse_method_descriptor(method_descriptor);

            if let Some(code_attribute) = method.get_code_attribute(&class) {
                let max_locals = code_attribute.max_locals() as usize;
                let max_stack = code_attribute.max_stack() as usize;
    
                let mut new_frame = Frame::new(max_locals, max_stack, 0, location);
    
                let nargs = types.iter().fold(0, |acc, x| if x == "J" || x == "D" { acc + 2 } else { acc + 1 }) - 1;
                for i in (0..nargs).rev() {
                    new_frame.locals[i] = current_frame.operands.pop()
                        .expect(&format!("Failed to resolve {}.{}", class_name, method_name));
                }

                self.frame_stack.push(new_frame);
                if first_time_loaded {
                    self.add_static_code_frame(&class);
                    self.add_string_pool_feed_frame(&class);
                }
                return;
            }

            if method.is_native() {
                let max_locals = types.len() - 1;
                let max_stack = 0;
                let mut new_frame = Frame::new(max_locals, max_stack, 0, location);
                for i in (0..max_locals).rev() {
                    new_frame.locals[i] = current_frame.operands.pop().unwrap();
                }
                let native_call = get_assoc_native_method(class_name, method_name, method_descriptor);
                native_call(self, &new_frame);
            }
        }
    }

    #[inline(always)]
    pub fn putstatic_op(&mut self, index: u16) {
        let frame_stack: &mut Vec<Box<Frame>> = &mut self.frame_stack;
        if let Some(current_frame) = frame_stack.last_mut() {
            let class = &current_frame.location.declaring_type;
            let field_const = class.get_constant(index as usize)
                .expect(&format!("could not find field constant at {index}"));
            let (class_name_index, name_and_type_index) = field_const.field_or_method_to_name_and_type();
            let name_and_type_constant = class.get_constant(name_and_type_index)
                .expect(&format!("could not find name and type at {index}"));
            let (field_name_index, descriptor_index) = name_and_type_constant.name_and_type_to_name_and_descriptor();

            let class_name = class.get_constant(class_name_index).unwrap().as_string();
            let field_name = class.get_constant(field_name_index).unwrap().as_string();
            let descriptor = class.get_constant(descriptor_index).unwrap().as_string();

            let value = current_frame.operands.pop().unwrap();
            let optional = self.classloader.find_loaded_class(class_name);
            let class: Rc<Class>;
            let mut first_time_loaded = false;
            if optional.is_none() {
                first_time_loaded = true;
                class = self.classloader.load_class(class_name);
            } else {
                class = optional.unwrap();
            }
            let field = class.find_field_with_name_and_descriptor(field_name, descriptor)
                .expect(&format!("could not find field: {field_name}:{descriptor}"));
            field.set_value(value);
            if first_time_loaded {
                self.add_static_code_frame(&class);
                self.add_string_pool_feed_frame(&class);
            }
        }
    }

    #[inline(always)]
    pub fn putfield_op(&mut self, index: u16) {
        let frame_stack: &mut Vec<Box<Frame>> = &mut self.frame_stack;
        if let Some(current_frame) = frame_stack.last_mut() {
            let class = &current_frame.location.declaring_type;
            let field_const = class.get_constant(index as usize)
                .expect(&format!("could not find field constant at {index}"));
            let (class_name_index, name_and_type_index) = field_const.field_or_method_to_name_and_type();
            let name_and_type_constant = class.get_constant(name_and_type_index)
                .expect(&format!("could not find name and type at {index}"));
            let (field_name_index, descriptor_index) = name_and_type_constant.name_and_type_to_name_and_descriptor();

            let class_name = class.get_constant(class_name_index).unwrap().as_string();
            let field_name = class.get_constant(field_name_index).unwrap().as_string();
            let descriptor = class.get_constant(descriptor_index).unwrap().as_string();

            let value = current_frame.operands.pop().unwrap();
            let objectref = current_frame.operands.pop().unwrap();
            let object = self.heap.get_object(objectref);
            let field: &mut Field = object.find_field_by_name_and_descriptor(field_name, descriptor)
                .expect(&format!("field: {field_name}:{descriptor} could not be found at {class_name}"));
            field.set_value(value);
        }
    }

    #[inline(always)]
    pub fn getfield_op(&mut self, index: u16) {
        let frame_stack: &mut Vec<Box<Frame>> = &mut self.frame_stack;
        if let Some(current_frame) = frame_stack.last_mut() {
            let class = &current_frame.location.declaring_type;
            let field_const = class.get_constant(index as usize)
                .expect(&format!("could not find field constant at {index}"));
            let (class_name_index, name_and_type_index) = field_const.field_or_method_to_name_and_type();
            let name_and_type_constant = class.get_constant(name_and_type_index)
                .expect(&format!("could not find name and type at {index}"));
            let (field_name_index, descriptor_index) = name_and_type_constant.name_and_type_to_name_and_descriptor();

            let class_name = class.get_constant(class_name_index).unwrap().as_string();
            let field_name = class.get_constant(field_name_index).unwrap().as_string();
            let descriptor = class.get_constant(descriptor_index).unwrap().as_string();

            let objectref = current_frame.operands.pop().unwrap();
            let object = self.heap.get_object(objectref);
            let field: &mut Field = object.find_field_by_name_and_descriptor(field_name, descriptor)
                .expect(&format!("field: {field_name}:{descriptor} could not be found at {class_name}"));
            current_frame.operands.push(field.value);
        }
    }

    #[inline(always)]
    pub fn getstatic_op(&mut self, index: u16) {
        let frame_stack: &mut Vec<Box<Frame>> = &mut self.frame_stack;
        if let Some(current_frame) = frame_stack.last_mut() {
            let class = &current_frame.location.declaring_type;
            let field_const = class.get_constant(index as usize)
                .expect(&format!("could not find field constant at {index}"));
            let (class_name_index, name_and_type_index) = field_const.field_or_method_to_name_and_type();
            let name_and_type_constant = class.get_constant(name_and_type_index)
            .expect(&format!("could not find name and type at {index}"));
            let (field_name_index, descriptor_index) = name_and_type_constant.name_and_type_to_name_and_descriptor();

            let class_name = class.get_constant(class_name_index).unwrap().as_string();
            let field_name = class.get_constant(field_name_index).unwrap().as_string();
            let descriptor = class.get_constant(descriptor_index).unwrap().as_string();

            let optional = self.classloader.find_loaded_class(class_name);
            let class: Rc<Class>;
            let mut first_time_loaded = false;
            if optional.is_none() {
                first_time_loaded = true;
                class = self.classloader.load_class(class_name);
            } else {
                class = optional.unwrap();
            }
                
            let field = class.find_field_with_name_and_descriptor(field_name, descriptor)
                .expect(&format!("could not find field: {field_name}:{descriptor}"));
            current_frame.operands.push(field.value.get());
            if first_time_loaded {
                self.add_string_pool_feed_frame(&class);
                self.add_static_code_frame(&class);
            }
        }
    }

    #[inline(always)]
    pub fn astore_op(&mut self, index: usize) {
        let frame_stack: &mut Vec<Box<Frame>> = &mut self.frame_stack;
        if let Some(current_frame) = frame_stack.last_mut() {
            let objectref = current_frame.operands.pop()
                .expect("failed to pop from operands stack");
            current_frame.locals[index] = objectref;
        }
    }

    #[inline(always)]
    pub fn iastore_op(&mut self) {
        let frame_stack: &mut Vec<Box<Frame>> = &mut self.frame_stack;
        if let Some(current_frame) = frame_stack.last_mut() {
            let value = current_frame.operands.pop().unwrap();
            let index = current_frame.operands.pop().unwrap();
            let arrayref = current_frame.operands.pop().unwrap();
            let array = self.heap.get_object(arrayref);
            let value = Value { i: value };
            array.set_array_value(index as usize, value);
        }
    }

    #[inline(always)]
    pub fn aload_op(&mut self, index: usize) {
        let frame_stack: &mut Vec<Box<Frame>> = &mut self.frame_stack;
        if let Some(current_frame) = frame_stack.last_mut() {
            let objectref = current_frame.locals[index];
            current_frame.operands.push(objectref);
        }
    }

    #[inline(always)]
    pub fn newarray_op(&mut self, atype: u8) {
        let frame_stack: &mut Vec<Box<Frame>> = &mut self.frame_stack;
        if let Some(current_frame) = frame_stack.last_mut() {
            let count = current_frame.operands.pop().unwrap();
            let arrayref = self.heap.allocate_array(atype, count as usize);
            current_frame.operands.push(arrayref);
        }
    }

    #[inline(always)]
    pub fn arraylength_op(&mut self) {
        let frame_stack: &mut Vec<Box<Frame>> = &mut self.frame_stack;
        if let Some(current_frame) = frame_stack.last_mut() {
            let arrayref = current_frame.operands.pop().unwrap();
            let array_object = self.heap.get_object(arrayref);
            if !array_object.is_array {
                panic!("object ref: {arrayref} is not an array");
            }
            current_frame.operands.push(array_object.get_array_length() as i32);
        }
    }

    #[inline(always)]
    pub fn castore_op(&mut self) {
        let frame_stack: &mut Vec<Box<Frame>> = &mut self.frame_stack;
        if let Some(current_frame) = frame_stack.last_mut() {
            let value = current_frame.operands.pop().unwrap();
            let index = current_frame.operands.pop().unwrap();
            let arrayref = current_frame.operands.pop().unwrap();
            let array_object = self.heap.get_object(arrayref);
            if !array_object.is_array {
                panic!("castore: not an array");
            }
            let value = Value { c: value as u16 };
            array_object.set_array_value(index as usize, value);
        }
    }

    #[inline(always)]
    pub fn caload_op(&mut self) {
        let frame_stack: &mut Vec<Box<Frame>> = &mut self.frame_stack;
        if let Some(current_frame) = frame_stack.last_mut() {
            let index = current_frame.operands.pop().unwrap();
            let arrayref = current_frame.operands.pop().unwrap();
            let array = self.heap.get_object(arrayref);
            if !array.is_array {
                panic!("caload: not an array");
            }
            let value = array.get_array_value(index as usize);
            unsafe { current_frame.operands.push(value.c as i32); }
        }
    }
    
    #[inline(always)]
    pub fn if_icmp_op<F: Fn(i32, i32) -> bool>(&mut self, branchoffset: i16, compare: F) {
        let frame_stack: &mut Vec<Box<Frame>> = &mut self.frame_stack;
        if let Some(current_frame) = frame_stack.last_mut() {
            let next_instr: i16 = current_frame.pc as i16 + branchoffset - 3;
            let value2 = current_frame.operands.pop().unwrap();
            let value1 = current_frame.operands.pop().unwrap();
            if compare(value1, value2) {
                current_frame.pc = next_instr as usize;
            }
        }
    }

    #[inline(always)]
    pub fn goto_op(&mut self, branchoffset: i16) {
        let frame_stack: &mut Vec<Box<Frame>> = &mut self.frame_stack;
        if let Some(current_frame) = frame_stack.last_mut() {
            let next_instr: i16 = current_frame.pc as i16 + branchoffset - 3;
            current_frame.pc = next_instr as usize;
        }
    }

    #[inline(always)]
    pub fn if_op<F: Fn(i32) -> bool>(&mut self, branchoffset: i16, compare: F) {
        let frame_stack: &mut Vec<Box<Frame>> = &mut self.frame_stack;
        if let Some(current_frame) = frame_stack.last_mut() {
            let value = current_frame.operands.pop().unwrap();
            if compare(value) {
                let next_instr: i16 = current_frame.pc as i16 + branchoffset - 3;
                current_frame.pc = next_instr as usize;
            }
        }
    }

    #[inline(always)]
    pub fn ifnonnull_op(&mut self, branchoffset: i16) {
        let frame_stack: &mut Vec<Box<Frame>> = &mut self.frame_stack;
        if let Some(current_frame) = frame_stack.last_mut() {
            let value = current_frame.operands.pop().unwrap();
            if value != 0 {
                let next_instr: i16 = current_frame.pc as i16 + branchoffset - 3;
                current_frame.pc = next_instr as usize;
            }
        }
    }

    #[inline(always)]
    pub fn ifnull_op(&mut self, branchoffset: i16) {
        let frame_stack: &mut Vec<Box<Frame>> = &mut self.frame_stack;
        if let Some(current_frame) = frame_stack.last_mut() {
            let value = current_frame.operands.pop().unwrap();
            if value == 0 {
                let next_instr: i16 = current_frame.pc as i16 + branchoffset - 3;
                current_frame.pc = next_instr as usize;
            }
        }
    }

    #[inline(always)]
    pub fn return_op(&mut self) {
        self.frame_stack.pop();
    }

    #[inline(always)]
    pub fn areturn_op(&mut self) {
        let frame_stack: &mut Vec<Box<Frame>> = &mut self.frame_stack;
        if let Some(current_frame) = frame_stack.last_mut() {
            let objectref = current_frame.operands.pop().unwrap();
            self.frame_stack.pop();
            let invoker_frame = self.frame_stack.last_mut().unwrap();
            invoker_frame.operands.push(objectref);
        }
    }

    #[inline(always)]
    pub fn ireturn_op(&mut self) {
        let frame_stack: &mut Vec<Box<Frame>> = &mut self.frame_stack;
        if let Some(current_frame) = frame_stack.last_mut() {
            let value = current_frame.operands.pop().unwrap();
            self.frame_stack.pop();
            let invoker_frame = self.frame_stack.last_mut().unwrap();
            invoker_frame.operands.push(value);
        }
    }
}
