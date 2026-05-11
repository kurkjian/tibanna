use std::collections::{HashMap, HashSet};

use crate::{
    backend::liveness::Liveness,
    common::graph::Graph,
    ir::types::{TIRFunction, VirtualRegister},
};

pub struct Allocation {
    pub allocations: HashMap<VirtualRegister, usize>,
    pub spilled: Vec<VirtualRegister>,
}

pub fn create_interference_graph(
    function: &TIRFunction,
    liveness: Liveness,
) -> Graph<VirtualRegister> {
    let mut graph = Graph::new();

    for block in function.blocks.iter() {
        let live = liveness
            .blocks
            .get(&block.label)
            .expect("Block liveness must exist");
        for p in &block.params {
            graph.add_vertex(*p);
            for v in &live.live_in {
                graph.add_edge(*p, *v);
            }
        }

        let mut live_out = live.live_out.clone();
        for instr in block.instructions.iter().rev() {
            graph.add_vertex(instr.dest);
            for v in &live_out {
                graph.add_edge(*v, instr.dest);
            }

            // We just def'ed the vreg; since we are going backwards, that value
            // is no longer live. We need the operands to be considered live as
            // we go back though
            live_out.remove(&instr.dest);
            for v in instr.uses() {
                live_out.insert(v);
            }
        }
    }

    graph
}

pub fn allocate_registers(graph: Graph<VirtualRegister>, num_registers: usize) -> Allocation {
    let mut allocations = HashMap::new();
    let mut spilled = Vec::new();
    let mut stack = Vec::with_capacity(graph.len());

    let mut work_graph = graph.clone();
    while !work_graph.is_empty() {
        let low_degree = work_graph
            .vertices()
            .find(|v| work_graph.degree(*v) < num_registers)
            .copied();

        match low_degree {
            Some(v) => {
                work_graph.remove_vertex(&v);
                stack.push(v);
            }
            None => {
                let max = work_graph
                    .vertices()
                    .max_by_key(|v| work_graph.degree(*v))
                    .copied()
                    .unwrap();

                work_graph.remove_vertex(&max);
                spilled.push(max);
                stack.push(max);
            }
        }
    }

    while let Some(v) = stack.pop() {
        let mut used_colors = HashSet::new();

        if let Some(neighbors) = graph.edges(&v) {
            for n in neighbors {
                if let Some(color) = allocations.get(n) {
                    used_colors.insert(*color);
                }
            }
        }

        let picked = (0..num_registers).find(|r| !used_colors.contains(r));
        match picked {
            Some(color) => {
                allocations.insert(v, color);
            }
            None => {
                spilled.push(v);
            }
        }
    }

    Allocation {
        allocations,
        spilled,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_valid_coloring(
        graph: &Graph<VirtualRegister>,
        alloc: &HashMap<VirtualRegister, usize>,
        num_registers: usize,
    ) {
        for &r in alloc.values() {
            assert!(r < num_registers, "invalid register assignment: {}", r);
        }

        for v in graph.vertices() {
            if let Some(neighbors) = graph.edges(v) {
                for n in neighbors {
                    if let (Some(&c1), Some(&c2)) = (alloc.get(v), alloc.get(n)) {
                        assert_ne!(c1, c2, "color conflict between {:?} and {:?}", v, n);
                    }
                }
            }
        }
    }

    #[test]
    fn test_single_node() {
        let mut g = Graph::new();
        g.add_vertex(VirtualRegister(1));

        let alloc = allocate_registers(g, 2);
        assert_eq!(alloc.allocations.len(), 1);
        assert!(alloc.allocations.contains_key(&VirtualRegister(1)));
    }

    #[test]
    fn test_chain_has_no_spills() {
        let mut g = Graph::new();
        g.add_edge(VirtualRegister(0), VirtualRegister(1));
        g.add_edge(VirtualRegister(1), VirtualRegister(2));
        g.add_edge(VirtualRegister(2), VirtualRegister(3));

        let alloc = allocate_registers(g.clone(), 2);
        assert_eq!(alloc.allocations.len(), 4);
        assert_valid_coloring(&g, &alloc.allocations, 2);
    }

    #[test]
    fn test_triangle_requires_spill() {
        let mut g = Graph::new();
        g.add_edge(VirtualRegister(0), VirtualRegister(1));
        g.add_edge(VirtualRegister(1), VirtualRegister(2));
        g.add_edge(VirtualRegister(2), VirtualRegister(0));

        let alloc = allocate_registers(g.clone(), 2);

        assert_eq!(alloc.spilled.len(), 2);
        assert_valid_coloring(&g, &alloc.allocations, 2);
    }

    #[test]
    fn test_independent_nodes() {
        let mut g = Graph::new();

        for i in 0..10 {
            g.add_vertex(VirtualRegister(i));
        }

        let alloc = allocate_registers(g, 2);
        assert_eq!(alloc.allocations.len(), 10);
    }
}
