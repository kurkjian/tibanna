use std::collections::HashMap;

use crate::{
    common::uf::UnionFind,
    ir::types::{BlockId, TIRBlock, TIRFunction, Terminator, VirtualRegister},
};

pub fn coalesce_registers(func: &TIRFunction) -> UnionFind<VirtualRegister> {
    let mut uf = UnionFind::new();
    let block_map: HashMap<BlockId, TIRBlock> = func
        .blocks
        .clone()
        .into_iter()
        .map(|b| (b.label, b))
        .collect();

    for block in &func.blocks {
        match &block.terminator {
            Terminator::Branch { target, params } => {
                let target_block = &block_map[target];
                coalesce_params(&mut uf, &target_block.params, params);
            }
            Terminator::ConditionalBranch {
                then_target,
                then_params,
                else_target,
                else_params,
                ..
            } => {
                let then_block = &block_map[then_target];
                let else_block = &block_map[else_target];
                coalesce_params(&mut uf, &then_block.params, then_params);
                coalesce_params(&mut uf, &else_block.params, else_params);
            }
            _ => {}
        }
    }

    uf
}

fn coalesce_params(
    uf: &mut UnionFind<VirtualRegister>,
    block_params: &[VirtualRegister],
    incoming: &[VirtualRegister],
) {
    for (dst, src) in block_params.iter().zip(incoming.iter()) {
        uf.union(*dst, *src);
    }
}
