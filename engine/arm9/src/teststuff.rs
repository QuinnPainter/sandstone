use alloc::string::ToString;
use ironds::display::console;

pub struct Obj1 {
    pub cntr: u32
}

impl crate::Component for Obj1 {
    fn start(&mut self) {
        self.cntr = 4;
    }

    fn update(&mut self) {
        self.cntr += 1;
        console::set_cursor_pos(1, 3);
        console::print(&self.cntr.to_string());
    }
}
