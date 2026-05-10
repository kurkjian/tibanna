#![allow(dead_code)]
use std::{collections::HashMap, fmt};

use crate::{
    backend::target::Target,
    ir::types::{Operation, TIRFunction, Terminator, VirtualRegister},
};

const EXIT_SYSCALL: usize = 60;
const WORD_SIZE: usize = 8;

enum CC {
    E,
    NE,
    G,
    GE,
    L,
    LE,
}

impl fmt::Display for CC {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CC::E => write!(f, "e"),
            CC::NE => write!(f, "ne"),
            CC::G => write!(f, "g"),
            CC::GE => write!(f, "ge"),
            CC::L => write!(f, "l"),
            CC::LE => write!(f, "le"),
        }
    }
}

enum Instruction {
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
    Xor(BinArgs),

    Jz(String),
    Jnz(String),
    Je(String),
    Jne(String),
    Jg(String),
    Jge(String),
    Jl(String),
    Jle(String),
    Jmp(String),

    Set(CC, Reg),

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
            Instruction::Set(cc, reg) => write!(f, "set{} {}", cc, reg.low_byte()),
            Instruction::And(args) => write!(f, "and {}", args),
            Instruction::Or(args) => write!(f, "or {}", args),
            Instruction::Xor(args) => write!(f, "xor {}", args),
            Instruction::Call(label) => write!(f, "call {}", label),
            Instruction::Ret => write!(f, "ret"),
        }
    }
}

struct MemRef {
    reg: Reg,
    offset: usize,
}

impl fmt::Display for MemRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}-{}]", self.reg, self.offset)
    }
}

enum BinArgs {
    ToReg(Reg, Arg64), //FIXME: i think this should actually be arg32
}

impl fmt::Display for BinArgs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinArgs::ToReg(reg, arg) => write!(f, "{}, {}", reg, arg),
        }
    }
}

enum MovArgs {
    ToReg(Reg, Arg64),
    ToMem(MemRef, Arg64), //FIXME: we can't mov [mem], [mem]; that needs to be invalid
}

impl fmt::Display for MovArgs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MovArgs::ToReg(reg, arg) => write!(f, "mov {}, {}", reg, arg),
            MovArgs::ToMem(mem, arg) => write!(f, "mov {}, {}", mem, arg),
        }
    }
}

enum Arg64 {
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

#[derive(Debug, strum_macros::Display, Copy, Clone)]
#[strum(serialize_all = "lowercase")]
enum Reg {
    Rax,
    Rbx,
    Rcx,
    Rdx,
    Rsi,
    Rdi,
    Rbp,
    Rsp,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
}

impl Reg {
    pub fn low_byte(&self) -> String {
        match self {
            Reg::Rax => "al".to_string(),
            Reg::Rbx => "bl".to_string(),
            Reg::Rcx => "cl".to_string(),
            Reg::Rdx => "dl".to_string(),
            Reg::Rsi => "sil".to_string(),
            Reg::Rdi => "dil".to_string(),
            Reg::Rbp => "bpl".to_string(),
            Reg::Rsp => "spl".to_string(),
            Reg::R8 => "r8b".to_string(),
            Reg::R9 => "r9b".to_string(),
            Reg::R10 => "r10b".to_string(),
            Reg::R11 => "r11b".to_string(),
            Reg::R12 => "r12b".to_string(),
            Reg::R13 => "r13b".to_string(),
            Reg::R14 => "r14b".to_string(),
            Reg::R15 => "r15b".to_string(),
        }
    }
}

impl From<usize> for Reg {
    fn from(val: usize) -> Self {
        match val {
            0 => Reg::Rax,
            1 => Reg::Rbx,
            2 => Reg::Rcx,
            3 => Reg::Rdx,
            4 => Reg::Rsi,
            5 => Reg::Rdi,
            6 => Reg::Rbp,
            7 => Reg::R8,
            8 => Reg::R9,
            9 => Reg::R10,
            10 => Reg::R11,
            11 => Reg::R12,
            12 => Reg::R13,
            13 => Reg::R14,
            14 => Reg::R15,
            _ => unreachable!(),
        }
    }
}

const CALLEE_SAVED: &[Reg] = &[
    Reg::Rbx,
    Reg::Rbp,
    Reg::R12,
    Reg::R13,
    Reg::R14,
    Reg::R15,
    Reg::Rsp,
];

const CALLER_SAVED: &[Reg] = &[
    Reg::Rax,
    Reg::Rbx,
    Reg::Rcx,
    Reg::Rdx,
    Reg::Rsi,
    Reg::Rdi,
    Reg::R8,
    Reg::R9,
    Reg::R10,
    Reg::R11,
];

#[derive(Default)]
pub struct X86_64 {
    instructions: Vec<Instruction>,
}

impl X86_64 {
    fn find_or_spill(
        &mut self,
        vr: VirtualRegister,
        alloc: &HashMap<VirtualRegister, usize>,
        scratch: Reg,
    ) -> (Reg, bool) {
        if let Some(reg) = alloc.get(&vr) {
            (Reg::from(*reg), false)
        } else {
            self.instructions.push(Instruction::Push(scratch));
            // todo: mov the spill loc to the scratch reg

            (scratch, true)
        }
    }

    fn restore_if_spilled(&mut self, reg: Reg, spilled: bool) {
        if spilled {
            self.instructions.push(Instruction::Pop(reg));
        }
    }

    fn store_dst(&mut self, dst: Option<&usize>, value: Arg64) {
        let mov = if let Some(reg) = dst {
            MovArgs::ToReg(Reg::from(*reg), value)
        } else {
            let mem = MemRef {
                reg: Reg::Rbp,
                offset: 0, // FIXME: Assign spills to a mem loc
            };
            MovArgs::ToMem(mem, value)
        };

        self.instructions.push(Instruction::Mov(mov));
    }

    fn emit_binop(
        &mut self,
        dst: Option<&usize>,
        vr1: VirtualRegister,
        vr2: VirtualRegister,
        alloc: &HashMap<VirtualRegister, usize>,
        op: fn(BinArgs) -> Instruction,
    ) {
        let (r1, spill1) = self.find_or_spill(vr1, alloc, Reg::Rax);
        let (r2, spill2) = self.find_or_spill(vr2, alloc, Reg::Rbx);

        self.instructions
            .push(op(BinArgs::ToReg(r1, Arg64::Reg(r2))));

        self.store_dst(dst, Arg64::Reg(r1));
        self.restore_if_spilled(Reg::Rbx, spill2);
        self.restore_if_spilled(Reg::Rax, spill1);
    }

    fn emit_cmp(
        &mut self,
        vr1: VirtualRegister,
        vr2: VirtualRegister,
        dst: VirtualRegister,
        alloc: &HashMap<VirtualRegister, usize>,
        cc: CC,
    ) {
        let (r1, spill1) = self.find_or_spill(vr1, alloc, Reg::Rax);
        let (r2, spill2) = self.find_or_spill(vr2, alloc, Reg::Rbx);
        let (dest, spill3) = self.find_or_spill(dst, alloc, Reg::Rcx);

        self.instructions
            .push(Instruction::Xor(BinArgs::ToReg(dest, Arg64::Reg(dest))));

        self.instructions
            .push(Instruction::Cmp(BinArgs::ToReg(r1, Arg64::Reg(r2))));

        self.instructions.push(Instruction::Set(cc, dest));
        self.restore_if_spilled(Reg::Rcx, spill3);
        self.restore_if_spilled(Reg::Rbx, spill2);
        self.restore_if_spilled(Reg::Rax, spill1);
    }

    fn translate_ir(
        &mut self,
        instr: crate::ir::types::Instruction,
        alloc: &(HashMap<VirtualRegister, usize>, Vec<VirtualRegister>),
    ) {
        let dst = alloc.0.get(&instr.dest);
        match instr.op {
            Operation::ConstInt(n) => {
                self.store_dst(dst, Arg64::Unsigned(n));
            }
            Operation::ConstBool(b) => {
                self.store_dst(dst, Arg64::Unsigned(b as usize));
            }
            Operation::Add(vr1, vr2) => {
                self.emit_binop(dst, vr1, vr2, &alloc.0, Instruction::Add);
            }
            Operation::Sub(vr1, vr2) => {
                self.emit_binop(dst, vr1, vr2, &alloc.0, Instruction::Sub);
            }
            Operation::Mul(vr1, vr2) => {
                self.emit_binop(dst, vr1, vr2, &alloc.0, Instruction::Mul);
            }
            Operation::And(vr1, vr2) => {
                self.emit_binop(dst, vr1, vr2, &alloc.0, Instruction::And);
            }
            Operation::Or(vr1, vr2) => {
                self.emit_binop(dst, vr1, vr2, &alloc.0, Instruction::Or);
            }
            Operation::Eq(vr1, vr2) => {
                self.emit_cmp(vr1, vr2, instr.dest, &alloc.0, CC::E);
            }
            Operation::Lt(vr1, vr2) => {
                self.emit_cmp(vr1, vr2, instr.dest, &alloc.0, CC::L);
            }
            Operation::Leq(vr1, vr2) => {
                self.emit_cmp(vr1, vr2, instr.dest, &alloc.0, CC::LE);
            }
            Operation::Gt(vr1, vr2) => {
                self.emit_cmp(vr1, vr2, instr.dest, &alloc.0, CC::G);
            }
            Operation::Geq(vr1, vr2) => {
                self.emit_cmp(vr1, vr2, instr.dest, &alloc.0, CC::GE);
            }
            Operation::Neq(vr1, vr2) => {
                self.emit_cmp(vr1, vr2, instr.dest, &alloc.0, CC::NE);
            }
            Operation::Call(function, vrs) => {
                if vrs.len() > 4 {
                    todo!("function call with more than 4 args. need to use stack")
                }

                let arg_regs = [Reg::Rbx, Reg::Rcx, Reg::Rdx, Reg::Rsi];
                for (arg, reg) in vrs.into_iter().zip(arg_regs) {
                    let loc = alloc.0.get(&arg);
                    if let Some(loc) = loc {
                        self.instructions.push(Instruction::Mov(MovArgs::ToReg(
                            reg,
                            Arg64::Reg(Reg::from(*loc)),
                        )));
                    } else {
                        let mem = MemRef {
                            reg: Reg::Rbp,
                            offset: 0, // FIXME: Assign spills to a mem loc
                        };
                        self.instructions
                            .push(Instruction::Mov(MovArgs::ToReg(reg, Arg64::Mem(mem))));
                    }
                }
                self.instructions.push(Instruction::Call(function));
            }
        }
    }

    fn translate_terminator(
        &mut self,
        function: &str,
        terminator: Terminator,
        alloc: &(HashMap<VirtualRegister, usize>, Vec<VirtualRegister>),
    ) {
        match terminator {
            Terminator::Void => unreachable!("should not have a void block"),
            Terminator::Exit(vr) => {
                let reg = alloc.0.get(&vr);
                if let Some(r) = reg {
                    self.instructions.push(Instruction::Mov(MovArgs::ToReg(
                        Reg::Rdi,
                        Arg64::Reg(Reg::from(*r)),
                    )));
                } else {
                    todo!("handle spill correctly: {}", vr)
                }

                self.instructions.push(Instruction::Mov(MovArgs::ToReg(
                    Reg::Rax,
                    Arg64::Unsigned(EXIT_SYSCALL),
                )));
                self.instructions.push(Instruction::Syscall);
            }
            Terminator::Return(vr) => {
                let reg = alloc.0.get(&vr);
                if let Some(r) = reg {
                    self.instructions.push(Instruction::Mov(MovArgs::ToReg(
                        Reg::Rax,
                        Arg64::Reg(Reg::from(*r)),
                    )));
                } else {
                    todo!("handle spill correctly")
                }
                self.instructions.push(Instruction::Ret);
            }
            Terminator::Branch { target, params: _ } => {
                let label = format!("{}_{}", function, target);
                self.instructions.push(Instruction::Jmp(label));
            }
            Terminator::ConditionalBranch {
                cond,
                then_target,
                then_params: _,
                else_target,
                else_params: _,
            } => {
                let then_label = format!("{}_{}", function, then_target);
                let else_label = format!("{}_{}", function, else_target);

                let r_cond = alloc.0.get(&cond);
                if let Some(reg) = r_cond {
                    self.instructions.push(Instruction::Cmp(BinArgs::ToReg(
                        Reg::from(*reg),
                        Arg64::Unsigned(1),
                    )));
                    self.instructions.push(Instruction::Je(then_label));
                    self.instructions.push(Instruction::Jmp(else_label));
                } else {
                    todo!("handle spills correctly");
                }
            }
        }
    }
}

impl Target for X86_64 {
    fn num_gp_registers(&self) -> usize {
        14
    }

    fn asm_header(&mut self) {
        self.instructions.extend(vec![
            Instruction::Directive("global".to_string(), "_start".to_string()),
            Instruction::Label("_start".to_string()),
            Instruction::Push(Reg::Rbp),
            Instruction::Mov(MovArgs::ToReg(Reg::Rbp, Arg64::Reg(Reg::Rsp))),
            Instruction::Call("main".to_string()),
        ]);
    }

    fn emit(
        &mut self,
        function: TIRFunction,
        alloc: (HashMap<VirtualRegister, usize>, Vec<VirtualRegister>),
    ) {
        // TODO: callee/caller save registers

        self.instructions
            .push(Instruction::Label(function.name.clone()));
        self.instructions.push(Instruction::Push(Reg::Rbp));
        self.instructions.push(Instruction::Mov(MovArgs::ToReg(
            Reg::Rbp,
            Arg64::Reg(Reg::Rsp),
        )));

        self.instructions.push(Instruction::Sub(BinArgs::ToReg(
            Reg::Rsp,
            Arg64::Unsigned(alloc.1.len() * WORD_SIZE),
        )));

        for block in function.blocks {
            let label = format!("{}_{}", function.name, block.label);
            self.instructions.push(Instruction::Label(label));

            for instr in block.instructions {
                self.translate_ir(instr, &alloc);
            }

            self.translate_terminator(&function.name, block.terminator, &alloc);
        }

        self.instructions.push(Instruction::Mov(MovArgs::ToReg(
            Reg::Rsp,
            Arg64::Reg(Reg::Rbp),
        )));
        self.instructions.push(Instruction::Pop(Reg::Rbp));
        self.instructions.push(Instruction::Ret);
    }

    fn asm(self) -> String {
        self.instructions
            .iter()
            .map(|instr| instr.to_string())
            .collect::<Vec<String>>()
            .join("\n")
    }
}
