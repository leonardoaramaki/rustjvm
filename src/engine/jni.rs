use std::mem::transmute;

use crate::types::{frame::Frame};
use super::Runtime;

/// Declare here native implementations of built-in methods.
fn java_lang_printstream_write(runtime: &mut Runtime, frame: &Frame) {
    print!("{}", runtime.get_string_from_obj(frame.locals[0]));
}

fn parse_descriptor(descriptor: &str, output: &mut Vec<String>) {
    let mut is_class = false;
    let mut is_array = false;
    let mut s: String = "".to_owned();
    for ch in descriptor.chars() {
        match ch {
            '[' => {
                is_array = true;
                s.push(ch);
            }
            'L' => {
                if !is_class {
                    is_class = true;
                } else {
                    s.push(ch);
                }
            }
            ';' => {
                is_class = false;
                is_array = false;
                output.push(s.clone());
                s.clear();
            }
            'Z' | 'C' | 'S' | 'I' | 'J' | 'F' | 'D' | 'V' => {
                if !is_class && !is_array {
                    output.push(ch.to_string());
                } else {
                    s.push(ch);
                    if !is_class && is_array {
                        is_array = false;
                        output.push(s.clone());
                        s.clear();
                    }
                }
            }
            _ => {
                if is_class {
                    s.push(ch);
                } else {
                    panic!();
                }
            }
        }
    }
}

/// Parse a method descriptor and return a `Vec<String>` where its elements are
/// the individual argument and return types.
/// 
/// # Examples
///```
/// let types: Vec<String> = parse_method_descriptor("foo(I[C)Ljava/lang/String;");
/// assert_eq!(types, ["I", "[C", "java/lang/String"]);
///```
pub fn parse_method_descriptor(descriptor: &str) -> Vec<String> {
    let mut types: Vec<String> = Vec::new();
    let mut descriptors = descriptor.split(')');
    let args_descriptor = &descriptors.next().unwrap()[1..];
    let return_descriptor = descriptors.next().unwrap();
    parse_descriptor(args_descriptor, types.as_mut());
    parse_descriptor(return_descriptor, types.as_mut());
    types
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_parse_arguments_correctly() {
        assert_eq!(parse_method_descriptor("(I)V"), ["I", "V"]);
        assert_eq!(parse_method_descriptor("(IJZ)Z"),  ["I", "J", "Z", "Z"]);
        assert_eq!(parse_method_descriptor("(Ljava/lang/String;)Z"), ["java/lang/String", "Z"]);
        assert_eq!(parse_method_descriptor("(JLjava/lang/String;)V"), ["J", "java/lang/String", "V"]);
        assert_eq!(parse_method_descriptor("(Ljava/lang/String;Z)V"), ["java/lang/String", "Z", "V"]);
        assert_eq!(parse_method_descriptor("([Ljava/lang/String;)V"), ["[java/lang/String", "V"]);
        assert_eq!(parse_method_descriptor("([F)V"), ["[F", "V"]);
        assert_eq!(parse_method_descriptor("(II)Ljava/lang/Object;"), ["I", "I", "java/lang/Object"]);
    }
}

pub fn get_assoc_native_method(class_name: &str, method_name: &str, descriptor: &str) -> fn(&mut Runtime, &Frame) -> () {
    let key = format!("{class_name}.{method_name}{descriptor}");
    match key.as_str() {
        "java/io/PrintStream.write(Ljava/lang/String;)V" => {
            let func = java_lang_printstream_write as *const();
            let func_ptr: fn(&mut Runtime, &Frame) -> () = unsafe { transmute(func) };
            return func_ptr;
        }
        _ => todo!()
    }
}
