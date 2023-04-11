use alloc::format;
use sandstone::node::{Node, NodeExtensionHandle};
use sandstone::{Script, ScriptContext};
use sandstone::ironds::input;
use sandstone::ironds::display::console;
use sandstone::hierarchy::HierarchyPoolTrait;
use sandstone::fixed::types::*;

const SPEED: I20F12 = I20F12::lit("3");

#[derive(Default)]
pub struct BallScript {
    x_vel: I20F12,
    y_vel: I20F12,
    moving: bool,
    player_score: u32,
    opponent_score: u32,
}

sandstone::register_script!(BallScript, 2);
impl Script for BallScript {
    fn start(&mut self, context: &mut ScriptContext) {
        self.start_ball(context.hierarchy.borrow_mut(context.handle));
        self.print_scores();
    }

    fn update(&mut self, context: &mut ScriptContext) {
        if self.moving {
            let node = context.hierarchy.borrow_mut(context.handle);

            // Check if ball went off top or bottom of screen
            if node.transform.y < 0 {
                self.player_score += 1;
                self.print_scores();
                self.start_ball(node);
            }
            if node.transform.y > 192*2 {
                self.opponent_score += 1;
                self.print_scores();
                self.start_ball(node);
            }

            // Check for collision
            let child_handle = node.child_handle.unwrap();
            let child = context.hierarchy.borrow(child_handle);
    
            let NodeExtensionHandle::RectCollider(collider_handle) = child.node_extension else {
                panic!("Ball doesn't have a collider");
            };
            let collider = context.hierarchy.borrow(collider_handle);
            for intersecting_node_handle in collider.intersect_list.iter() {
                let col_node = context.hierarchy.borrow(*intersecting_node_handle);
                if col_node.name.contains("Wall") {
                    // Collided with a wall - bounce horizontally
                    self.x_vel = -self.x_vel;
                } else {
                    // Collided with a paddle - bounce vertically
                    let node = context.hierarchy.borrow(context.handle);
                    if node.transform.y > 192 {
                        // on bottom screen - should bounce up
                        self.y_vel = I20F12::lit("-1");
                    } else {
                        // on top screen - should bounce down
                        self.y_vel = I20F12::lit("1");
                    }
                }
            }

            // Apply velocity
            let node = context.hierarchy.borrow_mut(context.handle);
            node.transform.x += self.x_vel * SPEED;
            node.transform.y += self.y_vel * SPEED;
        } else {
            let keys = input::read_keys();
            if keys.contains(input::Buttons::A) {
                self.moving = true;
                self.x_vel = I20F12::lit("1");
                self.y_vel = I20F12::lit("1");
            }
        }
    }
}

impl BallScript {
    fn start_ball (&mut self, node: &mut Node) {
        self.moving = false;
        node.transform.x = I20F12::lit("128");
        node.transform.y = I20F12::lit("192");
    }

    fn print_scores(&mut self) {
        console::set_cursor_pos(0, 0);
        console::print(&format!("{}", self.player_score));
        console::set_cursor_pos(30, 0);
        console::print(&format!("{: >2}", self.opponent_score));
    }
}
