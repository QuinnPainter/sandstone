#![no_std]
extern crate alloc;

pub mod player;
pub use player::PlayerPaddleScript;

pub mod ball;
pub use ball::BallScript;

pub mod opponent;
pub use opponent::AIPaddleScript;
