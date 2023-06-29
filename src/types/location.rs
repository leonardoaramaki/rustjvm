use super::{Class, MethodInfo};

#[derive(Debug)]
pub struct Location {
    pub declaring_type: Class,
    pub method: MethodInfo,
}

impl Location {
    pub fn new(class: &Class, method: &MethodInfo) -> Self {
        Self {
            declaring_type: class.clone(),
            method: method.clone(),
        }
    }
}
