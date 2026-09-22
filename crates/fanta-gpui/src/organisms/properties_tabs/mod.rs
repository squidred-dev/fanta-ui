//! Composable, headless tab organisms for a properties inspector.
mod code;
mod comments;
mod controls;
mod draw;
mod model;
mod motion;
mod prototype;
pub use code::CodeInspector;
pub use comments::CommentsInspector;
pub use draw::DrawInspector;
pub use model::*;
pub use motion::MotionInspector;
pub use prototype::PrototypeInspector;

#[cfg(test)]
mod interaction_tests;
