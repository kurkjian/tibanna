use std::collections::{HashMap, HashSet};

use crate::{
    backend::allocator::{
        coalesce::coalesce_registers, graph_color::{allocate_registers, create_interference_graph}, liveness::{Liveness, liveness},
    }, common::uf::UnionFind, ir::types::{BlockId, TIRFunction, VirtualRegister},
};

pub mod coalesce;
pub mod graph_color;
pub mod liveness;

#[derive(Debug)]
pub struct Allocation {
    allocations: HashMap<VirtualRegister, usize>,
    spilled: Vec<VirtualRegister>,
}

pub struct Allocator {
    liveness: Liveness,
    alloc: Allocation,
    uf: UnionFind<VirtualRegister>,
}

impl Allocator {
    pub fn new(function: &TIRFunction, num_registers: usize) -> Self {
        let liveness = liveness(function);
        let mut uf = coalesce_registers(function);
        let interference = create_interference_graph(function, &liveness, &mut uf);
        let alloc = allocate_registers(interference, num_registers);
        Self { liveness, alloc, uf }
    }

    pub fn location(&mut self, reg: VirtualRegister) -> Option<usize> {
        let canonical = self.uf.canonical(reg);
        self.alloc.allocations.get(&canonical).copied()
    }

    pub fn used(&self) -> Vec<usize> {
        self.alloc
            .allocations
            .values()
            .cloned()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>()
    }

    pub fn live_out(&self, id: BlockId, idx: usize) -> &HashSet<VirtualRegister> {
        &self.liveness.instr_liveness[&id][idx].live_after
    }

    pub fn spilled(&self) -> &[VirtualRegister] {
        &self.alloc.spilled
    }
}
