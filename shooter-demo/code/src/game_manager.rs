use alloc::format;
use sandstone::{Script, ScriptContext};
//use sandstone::fixed::types::*;
//use sandstone::hierarchy::HierarchyPoolTrait;
use sandstone::ironds::display::console;

#[derive(Default)]
pub struct GameManagerScript {
    score: u32,
}

sandstone::register_script!(GameManagerScript, 6);
impl Script for GameManagerScript {
    fn start(&mut self, _context: &mut ScriptContext) {
        sandstone::set_bg_colour(0x3A2E3F);
        self.draw_score();
    }

    fn update(&mut self, _context: &mut ScriptContext) {
    }
}

impl GameManagerScript {
    pub fn add_score(&mut self, amount: u32) {
        self.score += amount;
        self.draw_score();
    }

    fn draw_score(&self) {
        console::set_cursor_pos(0, 23);
        console::print(&format!("Score: {: <4}", self.score));
    }
}
