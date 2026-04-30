use std::fmt;

pub enum Instruction {
    Directive(String, String),
    Label(String),
    Syscall,

    Push(Reg),
    Pop(Reg),
    Mov(MovArgs),

    Add(BinArgs),
    Sub(BinArgs),
    Mul(BinArgs),
    Cmp(BinArgs),

    And(BinArgs),
    Or(BinArgs),

    Jz(String),
    Jnz(String),
    Je(String),
    Jne(String),
    Jg(String),
    Jge(String),
    Jl(String),
    Jle(String),
    Jmp(String),

    Call(String),
    Ret,
}

impl fmt::Debug for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::Directive(dir, val) => write!(f, "{} {}", dir, val),
            Instruction::Label(label) => write!(f, "{}:", label),
            Instruction::Syscall => write!(f, "syscall"),
            Instruction::Push(reg) => write!(f, "push {}", reg),
            Instruction::Pop(reg) => write!(f, "pop {}", reg),
            Instruction::Mov(mov) => write!(f, "{}", mov),
            Instruction::Add(args) => write!(f, "add {}", args),
            Instruction::Sub(args) => write!(f, "sub {}", args),
            Instruction::Mul(args) => write!(f, "imul {}", args),
            Instruction::Cmp(args) => write!(f, "cmp {}", args),
            Instruction::Jz(label) => write!(f, "jz {}", label),
            Instruction::Jnz(label) => write!(f, "jnz {}", label),
            Instruction::Je(label) => write!(f, "je {}", label),
            Instruction::Jne(label) => write!(f, "jne {}", label),
            Instruction::Jle(label) => write!(f, "jle {}", label),
            Instruction::Jl(label) => write!(f, "jl {}", label),
            Instruction::Jge(label) => write!(f, "jge {}", label),
            Instruction::Jg(label) => write!(f, "jg {}", label),
            Instruction::Jmp(label) => write!(f, "jmp {}", label),
            Instruction::And(args) => write!(f, "and {}", args),
            Instruction::Or(args) => write!(f, "or {}", args),
            Instruction::Call(label) => write!(f, "call {}", label),
            Instruction::Ret => write!(f, "ret"),
        }
    }
}

#[derive(Debug, strum_macros::Display, Copy, Clone)]
#[strum(serialize_all = "lowercase")]
pub enum Reg {
    Rsp,
    Rbp,
    Rax,
    Rbx,
    Rcx,
    Rdx,
    Rsi,
    Rdi,
}

pub struct MemRef {
    pub reg: Reg,
    pub offset: usize,
}

impl fmt::Display for MemRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}-{}]", self.reg, self.offset)
    }
}

pub enum BinArgs {
    ToReg(Reg, Arg64), //FIXME: i think this should actually be arg32
}

impl fmt::Display for BinArgs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinArgs::ToReg(reg, arg) => write!(f, "{}, {}", reg, arg),
        }
    }
}

pub enum MovArgs {
    ToReg(Reg, Arg64),
    ToMem(MemRef, Reg),
}

impl fmt::Display for MovArgs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MovArgs::ToReg(reg, arg) => write!(f, "mov {}, {}", reg, arg),
            MovArgs::ToMem(mem, reg) => write!(f, "mov {}, {}", mem, reg),
        }
    }
}

pub enum Arg64 {
    Reg(Reg),
    Unsigned(usize),
    Mem(MemRef),
}

impl fmt::Display for Arg64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Arg64::Reg(reg) => write!(f, "{}", reg),
            Arg64::Unsigned(val) => write!(f, "{}", val),
            Arg64::Mem(mem) => write!(f, "{}", mem),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mov_display() {
        let mov = Instruction::Mov(MovArgs::ToReg(Reg::Rax, Arg64::Reg(Reg::Rdi)));
        assert_eq!(format!("{}", mov), "mov rax, rdi");

        let mov = Instruction::Mov(MovArgs::ToReg(
            Reg::Rax,
            Arg64::Mem(MemRef {
                reg: Reg::Rbp,
                offset: 8,
            }),
        ));
        assert_eq!(format!("{}", mov), "mov rax, [rbp-8]");
    }
}
