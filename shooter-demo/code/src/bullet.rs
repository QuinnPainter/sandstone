use sandstone::node::Node;
use sandstone::pool::Handle;
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
        if node.transform.y < -64 {
            context.hierarchy.destroy_node(context.handle);
        }

        let node = context.hierarchy.borrow(context.handle);
        let child = context.hierarchy.borrow(node.child_handle.unwrap());

        let sandstone::node::NodeExtensionHandle::RectCollider(collider_handle) = child.node_extension else {
            panic!("Bullet has no Collider");
        };
        let mut hit_enemy_handle: Option<Handle<Node>> = None;
        let collider = context.hierarchy.borrow(collider_handle);
        for intersecting_node_handle in collider.intersect_list.iter() {
            if context.hierarchy.borrow(*intersecting_node_handle).name.contains("Enemy") {
                hit_enemy_handle = Some(*intersecting_node_handle);
            }
        }
        if let Some(hit_enemy_handle) = hit_enemy_handle {
            context.hierarchy.destroy_node(context.handle);
            let enemy_handle = context.hierarchy.borrow(hit_enemy_handle).parent_handle.unwrap();
            context.hierarchy.destroy_node(enemy_handle);
        }
    }
}
