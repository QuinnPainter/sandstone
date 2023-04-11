use sandstone::{Script, ScriptContext};
use sandstone::fixed::types::*;
use sandstone::hierarchy::HierarchyPoolTrait;

const SPEED: I20F12 = I20F12::lit("3");

#[derive(Default)]
pub struct EnemyScript {
    pub x_velocity: I20F12,
    pub y_velocity: I20F12,
}

sandstone::register_script!(EnemyScript, 5);
impl Script for EnemyScript {
    fn start(&mut self, _context: &mut ScriptContext) {
    }

    fn update(&mut self, context: &mut ScriptContext) {
        let node = context.hierarchy.borrow_mut(context.handle);
        node.transform.y += self.y_velocity * SPEED;
        node.transform.x += self.x_velocity * SPEED;

        if node.transform.y > 192*2 {
            context.hierarchy.destroy_node(context.handle);

            // Trigger game over
            let game_manager_handle = context.hierarchy.borrow(context.hierarchy.root).child_handle.unwrap();
            let game_manager = context.hierarchy.borrow_mut(game_manager_handle).cast_script_mut::<crate::GameManagerScript>();
            game_manager.game_over();
        }
    }
}
