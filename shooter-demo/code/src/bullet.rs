use sandstone::{Script, ScriptContext};
use sandstone::fixed::types::*;
use sandstone::hierarchy::HierarchyPoolTrait;

const BULLET_SPEED: I20F12 = I20F12::lit("5");

#[derive(Default)]
pub struct BulletScript {
}

sandstone::register_script!(BulletScript, 2);
impl Script for BulletScript {
    fn start(&mut self, _context: &mut ScriptContext) {
    }

    fn update(&mut self, context: &mut ScriptContext) {
        let node = context.hierarchy.borrow_mut(context.handle);
        node.transform.y -= BULLET_SPEED;

        let node = context.hierarchy.borrow(context.handle);
        let child = context.hierarchy.borrow(node.child_handle.unwrap());

        let collider_handle = if let sandstone::node::NodeExtensionHandle::RectCollider(n) = child.node_extension {
            n
        } else {
            panic!("");
        };
        let mut died = false;
        let collider = context.hierarchy.borrow(collider_handle);
        for intersecting_node_handle in collider.intersect_list.iter() {
            //ds::nocash::print(&context.hierarchy.borrow(*intersecting_node_handle).name);
            if context.hierarchy.borrow(*intersecting_node_handle).name.contains("Enemy") {
                //ds::nocash::print("died");
                died = true;
            }
        }
        if died {
            context.hierarchy.destroy_node(context.handle);
        }
    }
}
