use std::fmt;

pub enum Instruction {
    Syscall,
    Push,
    Pop,
    Mov(Mov),
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::Syscall => write!(f, "syscall"),
            Instruction::Push => write!(f, "push"),
            Instruction::Pop => write!(f, "pop"),
            Instruction::Mov(mov) => write!(f, "{}", mov),
        }
    }
}

#[derive(Debug, strum_macros::Display)]
#[strum(serialize_all = "lowercase")]
pub enum Reg {
    Rsp,
    Rbp,
    Rax,
    Rdi,
}

pub struct MemRef {
    pub reg: Reg,
    pub offset: usize,
}

impl fmt::Display for MemRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}+{}]", self.reg, self.offset)
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
                reg: Reg::Rsp,
                offset: 8,
            }),
        ));
        assert_eq!(format!("{}", mov), "mov rax, [rsp+8]");
    }
}
