use std::collections::HashMap;

use crate::ir::types::VirtualRegister;

pub mod coalesce;
pub mod graph_color;
pub mod liveness;

#[derive(Debug)]
pub struct Allocation {
    pub allocations: HashMap<VirtualRegister, usize>,
    pub spilled: Vec<VirtualRegister>,
}
