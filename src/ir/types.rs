use std::{collections::HashMap, fmt};

use crate::parser::{Expression, Statement};

pub enum Operation {
    // Consts
    ConstInt(usize),
    ConstBool(bool),

    // Binary Operations
    Add(VirtualRegister, VirtualRegister),
    Sub(VirtualRegister, VirtualRegister),
    Mul(VirtualRegister, VirtualRegister),
    Eq(VirtualRegister, VirtualRegister),
    Ne(VirtualRegister, VirtualRegister),
    Lt(VirtualRegister, VirtualRegister),
    Leq(VirtualRegister, VirtualRegister),
    Gt(VirtualRegister, VirtualRegister),
    Geq(VirtualRegister, VirtualRegister),
    Neq(VirtualRegister, VirtualRegister),
    And(VirtualRegister, VirtualRegister),
    Or(VirtualRegister, VirtualRegister),

    Call(String, Vec<VirtualRegister>),
}

pub struct Instruction {
    pub dest: VirtualRegister,
    pub op: Operation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VirtualRegister(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockId(pub usize);

impl fmt::Display for VirtualRegister {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "_v{}", self.0)
    }
}

impl fmt::Display for BlockId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "b{}", self.0)
    }
}

pub struct TIRBlock {
    pub label: BlockId,
    pub params: Vec<VirtualRegister>,
    pub instructions: Vec<Instruction>,
    pub terminator: Terminator,
}

pub struct TIRFunction {
    pub name: String,
    pub params: Vec<VirtualRegister>,
    pub blocks: Vec<TIRBlock>,
}

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

pub type Branch = (Option<Expression>, Vec<Statement>);
pub type Env = HashMap<String, VirtualRegister>;
