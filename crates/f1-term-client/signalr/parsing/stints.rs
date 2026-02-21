use f1_term_core::stint::{Stints, Stint};
use std::collections::HashMap;
use f1_term_core::driver::DriverNumber;
use serde_json::Value;

use super::Result;

// TODO
pub fn parse_stints(val: &Value) -> Result<HashMap<DriverNumber, Stints>> {
    let mut stints: HashMap<DriverNumber, Stints> = HashMap::new();
    for 
    Ok(stints)
}

fn parse_stint() -> Stint {
}
