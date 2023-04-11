use sandstone::{Script, ScriptContext};
use sandstone::fixed::types::*;
use sandstone::hierarchy::HierarchyPoolTrait;
use sandstone::ironds::input;

const MOVEMENT_SPEED: I20F12 = I20F12::lit("3");
const MOVEMENT_SPEED_WHILE_SHOOTING: I20F12 = I20F12::lit("1.5");
const SHOOT_COOLDOWN_RELOAD: u32 = 15;

#[derive(Default)]
pub struct PlayerScript {
    shoot_cooldown: u32,
}

sandstone::register_script!(PlayerScript, 1);
impl Script for PlayerScript {
    fn start(&mut self, _context: &mut ScriptContext) {
    }

    fn update(&mut self, context: &mut ScriptContext) {
        let node = context.hierarchy.borrow_mut(context.handle);

        let mut speed = MOVEMENT_SPEED;
        if self.shoot_cooldown > 0 {
            self.shoot_cooldown -= 1;
            speed = MOVEMENT_SPEED_WHILE_SHOOTING;
        }

        let keys = input::read_keys();
        if keys.contains(input::Buttons::UP) {
            node.transform.y -= speed;
        }
        if keys.contains(input::Buttons::DOWN) {
            node.transform.y += speed;
        }
        if keys.contains(input::Buttons::LEFT) {
            node.transform.x -= speed;
        }
        if keys.contains(input::Buttons::RIGHT) {
            node.transform.x += speed;
        }
        if keys.contains(input::Buttons::A) && self.shoot_cooldown == 0
        {
            let node = context.hierarchy.borrow(context.handle);
            let mut transform = node.transform;
            transform.x += I20F12::lit("12"); // center

            let handle = context.hierarchy.spawn_object(
                "Bullet", node.parent_handle.unwrap());
            let bullet = context.hierarchy.borrow_mut(handle);
            bullet.transform = transform;
            self.shoot_cooldown = SHOOT_COOLDOWN_RELOAD;
        }
    }
}
