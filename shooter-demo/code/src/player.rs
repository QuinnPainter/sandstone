use sandstone::{Script, ScriptContext};
use sandstone::fixed::types::*;
use sandstone::hierarchy::HierarchyPoolTrait;
use sandstone::ironds::input;

const SPEED: I20F12 = I20F12::lit("1.5");
const SHOOT_COOLDOWN_RELOAD: u32 = 10;

#[derive(Default)]
pub struct PlayerScript {
    shoot_cooldown: u32,
}

sandstone::register_script!(PlayerScript, 1);
impl Script for PlayerScript {
    fn start(&mut self, _context: &mut ScriptContext) {
        sandstone::set_bg_colour(0x3A2E3F);
    }

    fn update(&mut self, context: &mut ScriptContext) {
        let node = context.hierarchy.borrow_mut(context.handle);
        let keys = input::read_keys();
        let sandstone::node::NodeExtensionHandle::Sprite(s) = node.node_extension else {
            panic!("f");
        };
        let node_sprite = context.hierarchy.borrow_mut(s);
        let sandstone::node::sprite::SpriteType::Affine(aff) = &mut node_sprite.sprite_type else {
            panic!("f");
        };
        if keys.contains(input::Buttons::UP) {
            aff.scale_y += I20F12::lit("0.02");
            //node.transform.y -= SPEED;
        }
        if keys.contains(input::Buttons::DOWN) {
            aff.scale_y -= I20F12::lit("0.02");
            //node.transform.y += SPEED;
        }
        if keys.contains(input::Buttons::LEFT) {
            aff.scale_x -= I20F12::lit("0.02");
            //node.transform.x -= SPEED;
        }
        if keys.contains(input::Buttons::RIGHT) {
            aff.scale_x += I20F12::lit("0.02");
            //node.transform.x += SPEED;
        }
        if keys.contains(input::Buttons::L) {
            aff.rotation -= I20F12::lit("0.02");
        }
        if keys.contains(input::Buttons::R) {
            aff.rotation += I20F12::lit("0.02");
        }
        if keys.contains(input::Buttons::A) &&
            self.shoot_cooldown > SHOOT_COOLDOWN_RELOAD
        {
            let node = context.hierarchy.borrow(context.handle);
            let mut transform = node.transform;
            transform.x += I20F12::lit("12"); // center

            let handle = context.hierarchy.spawn_object(
                "Bullet", context.hierarchy.root);
            let bullet = context.hierarchy.borrow_mut(handle);
            bullet.transform = transform;
            self.shoot_cooldown = 0;
        }
        self.shoot_cooldown += 1;
    }
}
