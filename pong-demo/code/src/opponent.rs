use sandstone::node::Node;
use sandstone::pool::Handle;
use sandstone::{Script, ScriptContext};
use sandstone::hierarchy::HierarchyPoolTrait;
use sandstone::fixed::types::*;

const SPEED: I20F12 = I20F12::lit("1.5");

#[derive(Default)]
pub struct AIPaddleScript {
    ball_handle: Option<Handle<Node>>,
}

sandstone::register_script!(AIPaddleScript, 3);
impl Script for AIPaddleScript {
    fn start(&mut self, context: &mut ScriptContext) {
        let node = context.hierarchy.borrow(context.handle);
        self.ball_handle = context.hierarchy.find_by_name(node.parent_handle.unwrap(), "Ball");
    }

    fn update(&mut self, context: &mut ScriptContext) {
        let node = context.hierarchy.borrow(context.handle);
        let ball = context.hierarchy.borrow(self.ball_handle.unwrap());
        let going_right = node.transform.x < ball.transform.x;

        let node = context.hierarchy.borrow_mut(context.handle);
        if going_right {
            node.transform.x += SPEED;
        } else {
            node.transform.x -= SPEED;
        }
    }
}
