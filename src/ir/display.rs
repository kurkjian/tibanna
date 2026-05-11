use std::fmt;

use crate::ir::types::{
    BlockId, Instruction, Operation, TIRBlock, TIRFunction, Terminator, VirtualRegister,
};

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

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operation::ConstInt(n) => write!(f, "const_int {}", n),
            Operation::ConstBool(b) => write!(f, "const_bool {}", b),

            Operation::Add(a, b) => write!(f, "add {}, {}", a, b),
            Operation::Sub(a, b) => write!(f, "sub {}, {}", a, b),
            Operation::Mul(a, b) => write!(f, "mul {}, {}", a, b),

            Operation::And(a, b) => write!(f, "and {}, {}", a, b),
            Operation::Or(a, b) => write!(f, "or {}, {}", a, b),

            Operation::Lt(a, b) => write!(f, "lt {}, {}", a, b),
            Operation::Leq(a, b) => write!(f, "le {}, {}", a, b),
            Operation::Gt(a, b) => write!(f, "gt {}, {}", a, b),
            Operation::Geq(a, b) => write!(f, "ge {}, {}", a, b),
            Operation::Eq(a, b) => write!(f, "eq {}, {}", a, b),
            Operation::Neq(a, b) => write!(f, "ne {}, {}", a, b),

            Operation::Call(name, args) => {
                write!(f, "call {}(", name.name)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{arg}")?;
                }
                write!(f, ")")
            }
        }
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} = {}", self.dest, self.op)
    }
}

impl fmt::Display for Terminator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Terminator::Void => write!(f, "!! void"),
            Terminator::Branch { target, params } => {
                write!(f, "br b{}(", target.0)?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{p}")?;
                }
                write!(f, ")")
            }
            Terminator::ConditionalBranch {
                cond,
                then_target,
                then_params,
                else_target,
                else_params,
            } => {
                write!(f, "br_cond {}, b{}(", cond, then_target.0)?;
                for (i, p) in then_params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{p}")?;
                }

                write!(f, "), b{}(", else_target.0)?;
                for (i, p) in else_params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{p}")?;
                }

                write!(f, ")")
            }
            Terminator::Return(v) => write!(f, "return {}", v),
            Terminator::Exit(v) => write!(f, "exit {}", v),
        }
    }
}

impl fmt::Display for TIRBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "  b{}(", self.label.0)?;
        for (i, p) in self.params.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{p}")?;
        }
        writeln!(f, "):")?;

        for instr in &self.instructions {
            writeln!(f, "    {}", instr)?;
        }

        writeln!(f, "    {}", self.terminator)?;
        Ok(())
    }
}

impl fmt::Display for TIRFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "fn {}(", self.name)?;

        for (i, p) in self.params.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{p}")?;
        }

        writeln!(f, ") {{")?;

        for block in &self.blocks {
            writeln!(f, "{block}")?;
        }

        writeln!(f, "}}")
    }
}
