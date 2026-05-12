use std::collections::HashMap;

use crate::resolver::{FunctionId, ResolvedExpression, ResolvedStatement, SymbolId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operation {
    // Consts
    ConstInt(usize),
    ConstBool(bool),

    // Binary Operations
    Add(VirtualRegister, VirtualRegister),
    Sub(VirtualRegister, VirtualRegister),
    Mul(VirtualRegister, VirtualRegister),
    Eq(VirtualRegister, VirtualRegister),
    Lt(VirtualRegister, VirtualRegister),
    Leq(VirtualRegister, VirtualRegister),
    Gt(VirtualRegister, VirtualRegister),
    Geq(VirtualRegister, VirtualRegister),
    Neq(VirtualRegister, VirtualRegister),
    And(VirtualRegister, VirtualRegister),
    Or(VirtualRegister, VirtualRegister),

    Call(FunctionId, Vec<VirtualRegister>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instruction {
    pub dest: VirtualRegister,
    pub op: Operation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VirtualRegister(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockId(pub usize);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TIRBlock {
    pub label: BlockId,
    pub params: Vec<VirtualRegister>,
    pub instructions: Vec<Instruction>,
    pub terminator: Terminator,
}

#[derive(Debug)]
pub struct TIRFunction {
    pub name: String,
    pub params: Vec<VirtualRegister>,
    pub blocks: Vec<TIRBlock>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Terminator {
    Void,
    Exit(VirtualRegister),
    Return(VirtualRegister),
    Branch {
        target: BlockId,
        params: Vec<VirtualRegister>,
    },
    ConditionalBranch {
        cond: VirtualRegister,
        then_target: BlockId,
        then_params: Vec<VirtualRegister>,
        else_target: BlockId,
        else_params: Vec<VirtualRegister>,
    },
}

pub type Branch = (Option<ResolvedExpression>, Vec<ResolvedStatement>);
pub type Env = HashMap<SymbolId, VirtualRegister>;

pub struct PendingEdge {
    pub from: BlockId,
    pub params: Vec<VirtualRegister>,
}
