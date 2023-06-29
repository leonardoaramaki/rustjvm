use super::Location;

#[derive(Debug)]
pub struct Frame {
    pub max_locals: usize,
    pub max_stack: usize,
    pub pc: usize,
    pub location: Location,
    pub operands: Vec<i32>,
    pub locals: Vec<i32>,
}

impl Frame {
    pub fn new(max_locals: usize, max_stack: usize, pc: usize, location: Location) -> Box<Self> {
        let operands = Vec::new();
        let mut locals = Vec::with_capacity(max_locals as usize);
        for _ in 0..max_locals {
            locals.push(0);
        }
        Box::from(Self {
            max_locals,
            max_stack,
            pc,
            location,
            operands,
            locals,
        })
    }
}
