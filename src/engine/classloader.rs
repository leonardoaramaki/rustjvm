use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{types::Class, io};
#[derive(Debug)]
pub struct Classloader {
    loaded_classes: RefCell<HashMap<String, Rc<Class>>>
}

impl Classloader {
    pub fn new() -> Box<Self> {
        Box::from(
            Self {
                loaded_classes: RefCell::new(HashMap::new()),
            }
        )
    }

    pub fn set_class_as_loaded(&self, classname: &str, class: &Class) {
        self.loaded_classes.borrow_mut().insert(classname.to_string(), Rc::new(class.clone()));
    }

    /// Load class with binary name {classname} (`java/lang/Object`). First, checks if it has been
    /// already loaded by this classloader, otherwise initialize straight from the classpath.
    pub fn load_class(&self, classname: &str) -> Rc<Class> {
        let full_classname = format!("{}.class", classname);
        self.loaded_classes.borrow_mut().entry(classname.to_string())
            .or_insert_with(|| Rc::new(io::load_class_with_classpath("api", full_classname.as_str())));
        Rc::clone(self.loaded_classes.borrow().get(&classname.to_string()).unwrap())        
    }

    pub fn find_loaded_class(&self, classname: &str) -> Option<Rc<Class>> {
        let classes = self.loaded_classes.borrow();
        let class = classes.get(&classname.to_string());
        if class.is_some() {
            return Some(class.unwrap().clone());
        } else {
            return None;
        }
    }
}