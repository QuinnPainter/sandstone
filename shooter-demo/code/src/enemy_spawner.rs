use sandstone::{Script, ScriptContext};
use sandstone::fixed::types::*;
use sandstone::hierarchy::HierarchyPoolTrait;

#[derive(Default)]
pub struct EnemySpawnerScript {
    enemy_spawn_countdown: u32,
}

sandstone::register_script!(EnemySpawnerScript, 4);
impl Script for EnemySpawnerScript {
    fn start(&mut self, _context: &mut ScriptContext) {
        self.enemy_spawn_countdown = 20;
    }

    fn update(&mut self, context: &mut ScriptContext) {
        self.enemy_spawn_countdown -= 1;
        if self.enemy_spawn_countdown == 0 {
            spawn_enemy(context);
            self.enemy_spawn_countdown = 60;
        }
    }
}

fn spawn_enemy(context: &mut ScriptContext) {
    let new_enemy_x = sandstone::random::rand_i32_in_range(0, 256);
    let new_enemy_angle =
        I20F12::from_num(sandstone::random::rand_i32_in_range(-128, 128))
        / 128 // number from -1 to 1
        / 3;  // number from -0.333 to 0.333
    let new_enemy_handle = context.hierarchy.spawn_object("Enemy", context.handle);
    let new_enemy = context.hierarchy.borrow_mut(new_enemy_handle);
    new_enemy.transform.x = I20F12::from_num(new_enemy_x);
    new_enemy.transform.y = I20F12::lit("-32");

    let enemy_script = new_enemy.cast_script_mut::<crate::EnemyScript>();
    let (sin, cos) = sandstone::cordic::sin_cos(-new_enemy_angle);
    (enemy_script.x_velocity, enemy_script.y_velocity) = (sin, cos);

    let sandstone::node::NodeExtensionHandle::Sprite(sprite_handle) = new_enemy.node_extension else {
        panic!("Enemy had no Sprite");
    };
    let enemy_sprite = context.hierarchy.borrow_mut(sprite_handle);
    let sandstone::node::sprite::SpriteType::Affine(aff) = &mut enemy_sprite.sprite_type else {
        panic!("Enemy's Sprite was not affine");
    };
    aff.rotation = new_enemy_angle;
}

