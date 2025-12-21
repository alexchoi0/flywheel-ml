pub mod drift;
pub mod export;
pub mod graph;
pub mod health;
pub mod logs;
pub mod model;
pub mod stats;
pub mod validate;

pub struct Context {
    pub server: String,
    pub namespace: String,
    pub verbose: bool,
}
