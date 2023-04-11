use sandstone::{Script, ScriptContext};
use sandstone::fixed::types::*;
use sandstone::hierarchy::HierarchyPoolTrait;

#[derive(Default)]
pub struct EnemySpawnerScript {
}

sandstone::register_script!(EnemySpawnerScript, 4);
impl Script for EnemySpawnerScript {
    fn start(&mut self, _context: &mut ScriptContext) {
    }

    fn update(&mut self, context: &mut ScriptContext) {
    }
}
