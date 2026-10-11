//! Snowball War reconstruction library.
//!
//! The simulation modules (`game`, `participant`, `snowball`, `roam`,
//! `arena`, `data`, `input`) are SDL-free and fully testable; `render`
//! draws the recovered state with procedural graphics.

pub mod arena;
pub mod data;
pub mod font;
pub mod game;
pub mod input;
pub mod participant;
pub mod render;
pub mod roam;
pub mod snowball;
