use std::collections::{HashMap, HashSet};

use crate::{
    backend::allocator::{Allocation, liveness::Liveness},
    common::{graph::Graph, uf::UnionFind},
    ir::types::{TIRFunction, VirtualRegister},
};

pub fn create_interference_graph(
    function: &TIRFunction,
    liveness: Liveness,
    uf: &mut UnionFind<VirtualRegister>,
) -> Graph<VirtualRegister> {
    let mut graph = Graph::new();

    for block in function.blocks.iter() {
        let live = liveness
            .blocks
            .get(&block.label)
            .expect("Block liveness must exist");
        for p in &block.params {
            let p = uf.canonical(*p);
            graph.add_vertex(p);
            for v in &live.live_in {
                let v = uf.canonical(*v);
                graph.add_edge(p, v);
            }
        }

        let mut live_out = live.live_out.clone();

        let term_uses = block.terminator.uses();
        for v in &term_uses {
            let v = uf.canonical(*v);
            live_out.insert(v);
            for v2 in &term_uses {
                let v2 = uf.canonical(*v2);
                graph.add_edge(v, v2);
            }
        }

        for instr in block.instructions.iter().rev() {
            let dest = uf.canonical(instr.dest);
            graph.add_vertex(dest);
            for v in &live_out {
                let v = uf.canonical(*v);
                graph.add_edge(v, dest);
            }

            for v in instr.uses() {
                let v = uf.canonical(v);
                graph.add_edge(v, dest);
            }

            // We just def'ed the vreg; since we are going backwards, that value
            // is no longer live. We need the operands to be considered live as
            // we go back though
            live_out.remove(&dest);
            for v in instr.uses() {
                let v = uf.canonical(v);
                live_out.insert(v);
            }
        }
    }

    graph
}

pub fn allocate_registers(graph: Graph<VirtualRegister>, num_registers: usize) -> Allocation {
    let mut allocations = HashMap::new();
    let mut spilled = HashSet::new();
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
                spilled.insert(max);
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
                spilled.insert(v);
            }
        }
    }

    Allocation {
        allocations,
        spilled: spilled.into_iter().collect(),
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        backend::allocator::{coalesce::coalesce_registers, liveness::liveness},
        ir::types::{BlockId, Instruction, Operation, TIRBlock, Terminator},
    };

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

        assert_eq!(alloc.spilled.len(), 1);
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

    #[test]
    fn test_terminator_interference() {
        let f = TIRFunction {
            name: "test".to_string(),
            params: vec![],
            blocks: vec![
                TIRBlock {
                    label: BlockId(0),
                    params: vec![],
                    instructions: vec![
                        Instruction {
                            dest: VirtualRegister(0),
                            op: Operation::ConstInt(1),
                        },
                        Instruction {
                            dest: VirtualRegister(1),
                            op: Operation::ConstInt(2),
                        },
                        Instruction {
                            dest: VirtualRegister(2),
                            op: Operation::ConstInt(3),
                        },
                    ],
                    terminator: Terminator::ConditionalBranch {
                        cond: VirtualRegister(0),
                        then_target: BlockId(1),
                        then_params: vec![VirtualRegister(1)],
                        else_target: BlockId(2),
                        else_params: vec![VirtualRegister(2)],
                    },
                },
                TIRBlock {
                    label: BlockId(1),
                    params: vec![VirtualRegister(3)],
                    instructions: vec![],
                    terminator: Terminator::Return(VirtualRegister(1)),
                },
                TIRBlock {
                    label: BlockId(2),
                    params: vec![VirtualRegister(4)],
                    instructions: vec![],
                    terminator: Terminator::Return(VirtualRegister(2)),
                },
            ],
        };

        let l = liveness(&f);
        let mut uf = coalesce_registers(&f);
        let g = create_interference_graph(&f, l, &mut uf);

        assert_eq!(
            g.edges(&VirtualRegister(0)).unwrap(),
            &HashSet::from([VirtualRegister(1), VirtualRegister(2)])
        );
    }
}
