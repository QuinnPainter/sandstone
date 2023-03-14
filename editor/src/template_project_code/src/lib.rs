#![no_std]
extern crate alloc;

use sandstone::{Script, ScriptContext};

#[derive(Default)]
pub struct Obj1 {
    cntr: u32
}

sandstone::register_script!(Obj1, 1);
impl Script for Obj1 {
    fn start(&mut self, _context: &mut ScriptContext) {
        self.cntr = 4;
    }

    fn update(&mut self, _context: &mut ScriptContext) {
    	self.cntr += 1;
    }
}
