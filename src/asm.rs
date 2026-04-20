pub enum Instruction {
    Syscall,
    Push,
    Pop,
    Mov(Mov),
}

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

pub enum Mov {
    ToReg(Reg, Arg64),
}

pub enum Arg64 {
    Reg(Reg),
    Unsigned(usize),
    Mem(MemRef),
}
