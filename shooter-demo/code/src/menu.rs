use sandstone::{Script, ScriptContext};
use sandstone::ironds::{input, display::console};

#[derive(Default)]
pub struct MenuScript {
    start_counter: u64,
}

sandstone::register_script!(MenuScript, 3);
impl Script for MenuScript {
    fn start(&mut self, _context: &mut ScriptContext) {
        console::set_cursor_pos(11, 15);
        console::print("Press Start");
    }

    fn update(&mut self, context: &mut ScriptContext) {
        self.start_counter += 1;
        let keys = input::read_keys();
        if keys.contains(input::Buttons::A) || keys.contains(input::Buttons::START) {
            // Clear press start text
            console::set_cursor_pos(11, 15);
            console::print("           ");
            // Seed random number generator with the time since starting
            sandstone::random::seed(self.start_counter);
            context.hierarchy.set_scene("GameScene");
        }
    }
}
