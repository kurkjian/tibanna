use std::fmt;

pub enum Instruction {
    Syscall,
    Push(Reg),
    Pop(Reg),
    Mov(Mov),
    Add(Reg, Reg),
    Sub(Reg, Reg),
}

impl fmt::Debug for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::Syscall => write!(f, "syscall"),
            Instruction::Push(reg) => write!(f, "push {}", reg),
            Instruction::Pop(reg) => write!(f, "pop {}", reg),
            Instruction::Mov(mov) => write!(f, "{}", mov),
            Instruction::Add(reg1, reg2) => write!(f, "add {}, {}", reg1, reg2),
            Instruction::Sub(reg1, reg2) => write!(f, "sub {}, {}", reg1, reg2),
        }
    }
}

#[derive(Debug, strum_macros::Display)]
#[strum(serialize_all = "lowercase")]
pub enum Reg {
    Rsp,
    Rbp,
    Rax,
    Rbx,
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

pub enum Mov {
    ToReg(Reg, Arg64),
}

impl fmt::Display for Mov {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Mov::ToReg(reg, arg) => write!(f, "mov {}, {}", reg, arg),
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
        let mov = Instruction::Mov(Mov::ToReg(Reg::Rax, Arg64::Reg(Reg::Rdi)));
        assert_eq!(format!("{}", mov), "mov rax, rdi");

        let mov = Instruction::Mov(Mov::ToReg(
            Reg::Rax,
            Arg64::Mem(MemRef {
                reg: Reg::Rbp,
                offset: 8,
            }),
        ));
        assert_eq!(format!("{}", mov), "mov rax, [rbp-8]");
    }
}
