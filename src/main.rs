#![allow(dead_code)]
#![allow(unused_imports)]
mod prelude {
    pub use std::collections::HashMap;
    pub use std::{error::Error, io};
}

mod controller;
mod model;
mod ui;
use crate::prelude::*;

fn main() -> Result<(), Box<dyn Error>> {
    controller::run_app()?;
    Ok(())
}
