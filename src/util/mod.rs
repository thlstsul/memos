use nanoid::{alphabet, nanoid};

pub mod ast;

pub fn uuid() -> String {
    nanoid!(16, &alphabet::SAFE)
}
