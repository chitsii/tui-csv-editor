#![allow(dead_code)]
#![allow(unused_imports)]
mod prelude {
    pub use anyhow::Result;
    pub use std::collections::HashMap;
    pub use std::io;
}

mod controller;
mod model;
mod ui;
use crate::prelude::*;

fn main() -> Result<()> {
    controller::run_app()?;
    Ok(())
}
