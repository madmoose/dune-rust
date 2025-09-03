#![allow(clippy::identity_op)]
#![feature(file_buffered)]

mod framebuffer;
mod index_map;
mod palette;

pub use framebuffer::Framebuffer;
pub use index_map::IndexMap;
pub use palette::Palette;
