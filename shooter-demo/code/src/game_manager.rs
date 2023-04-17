use alloc::format;
use sandstone::{Script, ScriptContext};
use sandstone::ironds::display::console;
use sandstone::ironds::input;

const DISPLAY_START_MESSAGE_TIME: u32 = 120;
const START_MESSAGE: &str = "Don't let any get past!";
const GAMEOVER_MESSAGE: &str = "Game Over\n\n      Press Start to retry";

#[derive(Default, PartialEq, Eq, Clone, Copy)]
enum GameState {
    #[default]
    Start,
    Playing,
    GameOver,
}

#[derive(Default)]
pub struct GameManagerScript {
    score: u32,
    state: GameState,
    start_timer: u32,
}

sandstone::register_script!(GameManagerScript, 6);
impl Script for GameManagerScript {
    fn start(&mut self, _context: &mut ScriptContext) {
        sandstone::set_bg_colour(0x3A2E3F);
        self.start_timer = DISPLAY_START_MESSAGE_TIME;
        self.draw_score();
        console::set_cursor_pos(4, 10);
        console::print(START_MESSAGE);
    }

    fn update(&mut self, context: &mut ScriptContext) {
        if self.state == GameState::Start {
            self.start_timer -= 1;
            if self.start_timer == 0 {
                self.state = GameState::Playing;
                console::set_cursor_pos(4, 10);
                console::print(&" ".repeat(START_MESSAGE.len()));
            }
        }
        if self.state == GameState::GameOver {
            if input::read_keys().contains(input::Buttons::START) {
                console::set_cursor_pos(12, 10);
                console::print(&" ".repeat(32*3));
                context.hierarchy.set_scene("GameScene");
            }
        }
    }
}

impl GameManagerScript {
    pub fn game_over(&mut self) {
        if self.state != GameState::GameOver {
            console::set_cursor_pos(12, 10);
            console::print(GAMEOVER_MESSAGE);
            self.state = GameState::GameOver;
        }
    }

    pub fn add_score(&mut self, amount: u32) {
        if self.state != GameState::GameOver {
            self.score += amount;
            self.draw_score();
        }
    }

    fn draw_score(&self) {
        console::set_cursor_pos(0, 23);
        console::print(&format!("Score: {: <4}", self.score));
    }
}
