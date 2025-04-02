pub mod ast;
pub mod md;

use nanoid::{alphabet, nanoid};

pub fn uuid() -> String {
    nanoid!(16, &alphabet::SAFE)
}
