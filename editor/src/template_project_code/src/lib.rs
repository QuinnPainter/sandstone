#![no_std]
extern crate alloc;

use core::num::NonZeroU32;
use alloc::string::ToString;
use sandstone::{Script, ScriptContext};
use sandstone::ironds::display::console;

#[derive(Default)]
pub struct Obj1 {
    cntr: u32
}

impl sandstone::hierarchy::HasTypeId for Obj1 {
    fn type_id() -> NonZeroU32 {
        NonZeroU32::new(2).unwrap()
    }
}

impl Script for Obj1 {
    fn start(&mut self, _context: &mut ScriptContext) {
        self.cntr = 4;
    }

    fn update(&mut self, _context: &mut ScriptContext) {
    	self.cntr += 1;
        console::set_cursor_pos(1, 3);
        console::print(&self.cntr.to_string());
    }
}
