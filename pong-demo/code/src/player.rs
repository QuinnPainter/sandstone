use sandstone::{Script, ScriptContext};
use sandstone::ironds::input;
use sandstone::hierarchy::HierarchyPoolTrait;
use sandstone::fixed::types::*;

const SPEED: I20F12 = I20F12::lit("3");

#[derive(Default)]
pub struct PlayerPaddleScript {
}

sandstone::register_script!(PlayerPaddleScript, 1);
impl Script for PlayerPaddleScript {
    fn start(&mut self, _context: &mut ScriptContext) {
    }

    fn update(&mut self, context: &mut ScriptContext) {
        let node = context.hierarchy.borrow_mut(context.handle);
        let keys = input::read_keys();
        if keys.contains(input::Buttons::LEFT) {
            node.transform.x -= SPEED;
        }
        if keys.contains(input::Buttons::RIGHT) {
            node.transform.x += SPEED;
        }
    }
}
