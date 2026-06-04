#![allow(dead_code)]
use std::{collections::HashMap, fmt};

use crate::{
    backend::{allocator::Allocator, target::Target, targets::peephole::peephole},
    ir::{
        self,
        types::{BlockId, Operation, TIRBlock, TIRFunction, Terminator, VirtualRegister},
    },
};

const EXIT_SYSCALL: usize = 60;
const WORD_SIZE: usize = 8;

struct RegisterMap {
    /// virtual register -> physical register id
    allocator: Allocator,
    /// virtual register -> spilled offset from rbp
    spilled: HashMap<VirtualRegister, usize>,
}

impl RegisterMap {
    fn operand(&mut self, vr: VirtualRegister) -> Operand {
        let loc = self.allocator.location(vr);
        if let Some(reg) = loc {
            Operand::Reg(Reg::from(reg))
        } else {
            let offset = self.spilled.get(&vr).expect("vr must have been spilled");
            Operand::Mem(MemRef {
                reg: Reg::Rbp,
                offset: *offset,
            })
        }
    }
}

pub enum CC {
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

pub enum Instruction {
    Directive(String, String),
    Label(String),
    Syscall,

    Push(Reg),
    Pop(Reg),
    Mov(Operand, Operand),

    Add(Operand, Operand),
    Sub(Operand, Operand),
    Mul(Operand, Operand),
    Cmp(Operand, Operand),

    And(Operand, Operand),
    Or(Operand, Operand),
    Xor(Operand, Operand),

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
            Instruction::Syscall => write!(f, "    syscall"),
            Instruction::Push(reg) => write!(f, "    push {}", reg),
            Instruction::Pop(reg) => write!(f, "    pop {}", reg),
            Instruction::Mov(op1, op2) => write!(f, "    mov {}, {}", op1, op2),
            Instruction::Add(op1, op2) => write!(f, "    add {}, {}", op1, op2),
            Instruction::Sub(op1, op2) => write!(f, "    sub {}, {}", op1, op2),
            Instruction::Mul(op1, op2) => write!(f, "    imul {}, {}", op1, op2),
            Instruction::Cmp(op1, op2) => write!(f, "    cmp {}, {}", op1, op2),
            Instruction::Jz(label) => write!(f, "    jz {}", label),
            Instruction::Jnz(label) => write!(f, "    jnz {}", label),
            Instruction::Je(label) => write!(f, "    je {}", label),
            Instruction::Jne(label) => write!(f, "    jne {}", label),
            Instruction::Jle(label) => write!(f, "    jle {}", label),
            Instruction::Jl(label) => write!(f, "    jl {}", label),
            Instruction::Jge(label) => write!(f, "    jge {}", label),
            Instruction::Jg(label) => write!(f, "    jg {}", label),
            Instruction::Jmp(label) => write!(f, "    jmp {}", label),
            Instruction::Set(cc, reg) => write!(f, "    set{} {}", cc, reg.low_byte()),
            Instruction::And(op1, op2) => write!(f, "    and {}, {}", op1, op2),
            Instruction::Or(op1, op2) => write!(f, "    or {}, {}", op1, op2),
            Instruction::Xor(op1, op2) => write!(f, "    xor {}, {}", op1, op2),
            Instruction::Call(label) => write!(f, "    call {}", label),
            Instruction::Ret => write!(f, "    ret"),
        }
    }
}

pub struct MemRef {
    reg: Reg,
    offset: usize,
}

impl fmt::Display for MemRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}-{}]", self.reg, self.offset)
    }
}

pub enum Operand {
    Reg(Reg),
    Imm(usize),
    Mem(MemRef),
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operand::Reg(reg) => write!(f, "{}", reg),
            Operand::Imm(val) => write!(f, "{}", val),
            Operand::Mem(mem) => write!(f, "{}", mem),
        }
    }
}

#[derive(Debug, strum_macros::Display, Copy, Clone, PartialEq, Eq)]
#[strum(serialize_all = "lowercase")]
pub enum Reg {
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
            // 2 => Reg::Rbp,
            6 => Reg::R8,
            7 => Reg::R9,
            8 => Reg::R10,
            9 => Reg::R11,
            10 => Reg::R12,
            11 => Reg::R13,
            12 => Reg::R14,
            13 => Reg::R15,
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
        alloc: &mut RegisterMap,
        scratch: Reg,
    ) -> (Reg, bool) {
        if let Some(reg) = alloc.allocator.location(vr) {
            (Reg::from(reg), false)
        } else {
            self.instructions.push(Instruction::Push(scratch));
            let offset = alloc.spilled.get(&vr).expect("vr must have been spilled");
            let mem = MemRef {
                reg: Reg::Rbp,
                offset: *offset,
            };
            self.instructions
                .push(Instruction::Mov(Operand::Reg(scratch), Operand::Mem(mem)));
            (scratch, true)
        }
    }

    fn restore_if_spilled(
        &mut self,
        reg: Reg,
        spilled: bool,
        alloc: &RegisterMap,
        vr: VirtualRegister,
    ) {
        if spilled {
            let offset = alloc.spilled.get(&vr).expect("reg must have been spilled");
            let mem = MemRef {
                reg: Reg::Rbp,
                offset: *offset,
            };
            self.instructions
                .push(Instruction::Mov(Operand::Mem(mem), Operand::Reg(reg)));
            self.instructions.push(Instruction::Pop(reg));
        }
    }

    fn store_dst(&mut self, dst: VirtualRegister, value: Operand, alloc: &mut RegisterMap) {
        let operand = alloc.operand(dst);
        self.instructions.push(Instruction::Mov(operand, value));
    }

    fn save_caller_regs(&mut self, alloc: &RegisterMap) {
        let mut used = alloc.allocator.used();
        used.sort();

        for reg in used {
            let r = Reg::from(reg);
            if CALLER_SAVED.contains(&r) {
                self.instructions.push(Instruction::Push(r));
            }
        }
    }

    fn pop_caller_regs(&mut self, alloc: &RegisterMap) {
        let mut used = alloc.allocator.used();
        used.sort();

        for reg in used.iter().rev() {
            let r = Reg::from(*reg);
            if CALLER_SAVED.contains(&r) {
                self.instructions.push(Instruction::Pop(r));
            }
        }
    }

    fn emit_binop(
        &mut self,
        dst: VirtualRegister,
        vr1: VirtualRegister,
        vr2: VirtualRegister,
        alloc: &mut RegisterMap,
        op: fn(Operand, Operand) -> Instruction,
    ) {
        let (r1, spill1) = self.find_or_spill(vr1, alloc, Reg::Rax);
        let (r2, spill2) = self.find_or_spill(vr2, alloc, Reg::R8);

        self.instructions
            .push(op(Operand::Reg(r1), Operand::Reg(r2)));

        self.store_dst(dst, Operand::Reg(r1), alloc);
        self.restore_if_spilled(Reg::R8, spill2, alloc, vr2);
        self.restore_if_spilled(Reg::Rax, spill1, alloc, vr1);
    }

    fn emit_cmp(
        &mut self,
        vr1: VirtualRegister,
        vr2: VirtualRegister,
        dst: VirtualRegister,
        alloc: &mut RegisterMap,
        cc: CC,
    ) {
        let (r1, spill1) = self.find_or_spill(vr1, alloc, Reg::Rax);
        let (r2, spill2) = self.find_or_spill(vr2, alloc, Reg::R8);
        let (dest, spill3) = self.find_or_spill(dst, alloc, Reg::R9);
        self.instructions
            .push(Instruction::Xor(Operand::Reg(dest), Operand::Reg(dest)));
        self.instructions
            .push(Instruction::Cmp(Operand::Reg(r1), Operand::Reg(r2)));
        self.instructions.push(Instruction::Set(cc, dest));
        self.restore_if_spilled(Reg::R9, spill3, alloc, dst);
        self.restore_if_spilled(Reg::R8, spill2, alloc, vr2);
        self.restore_if_spilled(Reg::Rax, spill1, alloc, vr1);
    }

    fn translate_ir(&mut self, instr: ir::types::Instruction, alloc: &mut RegisterMap) {
        let dst = instr.dest;
        match instr.op {
            Operation::ConstInt(n) => {
                self.store_dst(dst, Operand::Imm(n), alloc);
            }
            Operation::ConstBool(b) => {
                self.store_dst(dst, Operand::Imm(b as usize), alloc);
            }
            Operation::Add(vr1, vr2) => {
                self.emit_binop(dst, vr1, vr2, alloc, Instruction::Add);
            }
            Operation::Sub(vr1, vr2) => {
                self.emit_binop(dst, vr1, vr2, alloc, Instruction::Sub);
            }
            Operation::Mul(vr1, vr2) => {
                self.emit_binop(dst, vr1, vr2, alloc, Instruction::Mul);
            }
            Operation::And(vr1, vr2) => {
                self.emit_binop(dst, vr1, vr2, alloc, Instruction::And);
            }
            Operation::Or(vr1, vr2) => {
                self.emit_binop(dst, vr1, vr2, alloc, Instruction::Or);
            }
            Operation::Eq(vr1, vr2) => {
                self.emit_cmp(vr1, vr2, instr.dest, alloc, CC::E);
            }
            Operation::Lt(vr1, vr2) => {
                self.emit_cmp(vr1, vr2, instr.dest, alloc, CC::L);
            }
            Operation::Leq(vr1, vr2) => {
                self.emit_cmp(vr1, vr2, instr.dest, alloc, CC::LE);
            }
            Operation::Gt(vr1, vr2) => {
                self.emit_cmp(vr1, vr2, instr.dest, alloc, CC::G);
            }
            Operation::Geq(vr1, vr2) => {
                self.emit_cmp(vr1, vr2, instr.dest, alloc, CC::GE);
            }
            Operation::Neq(vr1, vr2) => {
                self.emit_cmp(vr1, vr2, instr.dest, alloc, CC::NE);
            }
            Operation::Call(function, vrs) => {
                if vrs.len() > 4 {
                    todo!("function call with more than 4 args. need to use stack")
                }
                self.save_caller_regs(alloc);

                let arg_regs = [Reg::Rbx, Reg::Rcx, Reg::Rdx, Reg::Rsi];
                for (arg, reg) in vrs.into_iter().zip(arg_regs) {
                    let operand = alloc.operand(arg);
                    self.instructions
                        .push(Instruction::Mov(Operand::Reg(reg), operand));
                }
                self.instructions.push(Instruction::Call(function.name));
                self.store_dst(dst, Operand::Reg(Reg::Rax), alloc);
                self.pop_caller_regs(alloc);
            }
        }
    }

    fn translate_terminator(
        &mut self,
        function: &str,
        terminator: Terminator,
        alloc: &mut RegisterMap,
        blocks: &HashMap<BlockId, TIRBlock>,
    ) {
        match terminator {
            Terminator::Void => unreachable!("should not have a void block"),
            Terminator::Exit(vr) => {
                let operand = alloc.operand(vr);
                self.instructions
                    .push(Instruction::Mov(Operand::Reg(Reg::Rdi), operand));

                self.instructions.push(Instruction::Mov(
                    Operand::Reg(Reg::Rax),
                    Operand::Imm(EXIT_SYSCALL),
                ));
                self.instructions.push(Instruction::Syscall);
            }
            Terminator::Return(vr) => {
                let operand = alloc.operand(vr);
                self.instructions
                    .push(Instruction::Mov(Operand::Reg(Reg::Rax), operand));

                self.instructions.push(Instruction::Mov(
                    Operand::Reg(Reg::Rsp),
                    Operand::Reg(Reg::Rbp),
                ));
                self.instructions.push(Instruction::Pop(Reg::Rbp));
                self.instructions.push(Instruction::Ret);
            }
            Terminator::Branch { target, params } => {
                let label = format!("{}_{}", function, target);
                let t = &blocks[&target];
                for (param, arg) in params.iter().zip(t.params.iter()) {
                    // i dont think these spills matter because param/arg are
                    // actually the same value, just moved around?
                    let (r1, _) = self.find_or_spill(*param, alloc, Reg::Rax);
                    let (r2, _) = self.find_or_spill(*arg, alloc, Reg::Rax);
                    self.instructions
                        .push(Instruction::Mov(Operand::Reg(r2), Operand::Reg(r1)));
                }

                self.instructions.push(Instruction::Jmp(label));
            }
            Terminator::ConditionalBranch {
                cond,
                then_target,
                then_params,
                else_target,
                else_params,
            } => {
                let then_label = format!("{}_{}", function, then_target);
                let else_label = format!("{}_{}", function, else_target);

                let t = &blocks[&then_target];
                for (param, arg) in then_params.iter().zip(t.params.iter()) {
                    // i dont think these spills matter because param/arg are
                    // actually the same value, just moved around?
                    let (r1, _) = self.find_or_spill(*param, alloc, Reg::Rax);
                    let (r2, _) = self.find_or_spill(*arg, alloc, Reg::Rax);
                    self.instructions
                        .push(Instruction::Mov(Operand::Reg(r2), Operand::Reg(r1)));
                }

                let t = &blocks[&else_target];
                for (param, arg) in else_params.iter().zip(t.params.iter()) {
                    // i dont think these spills matter because param/arg are
                    // actually the same value, just moved around?
                    let (r1, _) = self.find_or_spill(*param, alloc, Reg::Rax);
                    let (r2, _) = self.find_or_spill(*arg, alloc, Reg::Rax);
                    self.instructions
                        .push(Instruction::Mov(Operand::Reg(r2), Operand::Reg(r1)));
                }

                let operand = alloc.operand(cond);
                self.instructions
                    .push(Instruction::Cmp(operand, Operand::Imm(0)));
                self.instructions.push(Instruction::Jne(then_label));
                self.instructions.push(Instruction::Jmp(else_label));
            }
        }
    }
}

impl Target for X86_64 {
    fn num_gp_registers(&self) -> usize {
        13
    }

    fn asm_header(&mut self) {
        self.instructions.extend(vec![
            Instruction::Directive("global".to_string(), "_start".to_string()),
            Instruction::Label("_start".to_string()),
            Instruction::Push(Reg::Rbp),
            Instruction::Mov(Operand::Reg(Reg::Rbp), Operand::Reg(Reg::Rsp)),
            Instruction::Call("main".to_string()),
        ]);
    }

    fn emit(&mut self, function: TIRFunction, allocator: Allocator) {
        let mut spilled = HashMap::with_capacity(allocator.spilled().len());
        for (i, vr) in allocator.spilled().iter().enumerate() {
            spilled.insert(*vr, (i + 1) * WORD_SIZE);
        }
        let mut alloc = RegisterMap { allocator, spilled };

        // TODO: callee/caller save registers

        self.instructions
            .push(Instruction::Label(function.name.clone()));
        self.instructions.push(Instruction::Push(Reg::Rbp));
        self.instructions.push(Instruction::Mov(
            Operand::Reg(Reg::Rbp),
            Operand::Reg(Reg::Rsp),
        ));

        self.instructions.push(Instruction::Sub(
            Operand::Reg(Reg::Rsp),
            Operand::Imm(alloc.spilled.len() * WORD_SIZE),
        ));

        let arg_regs = [Reg::Rbx, Reg::Rcx, Reg::Rdx, Reg::Rsi];
        for (p, reg) in function.params.iter().zip(arg_regs) {
            let operand = alloc.operand(*p);
            self.instructions
                .push(Instruction::Mov(Operand::Reg(reg), operand));
        }

        let blocks = function
            .blocks
            .clone()
            .into_iter()
            .map(|b| (b.label, b))
            .collect();
        for block in function.blocks {
            let label = format!("{}_{}", function.name, block.label);
            self.instructions.push(Instruction::Label(label));

            for instr in block.instructions {
                self.translate_ir(instr, &mut alloc);
            }

            self.translate_terminator(&function.name, block.terminator, &mut alloc, &blocks);
        }

        self.instructions.push(Instruction::Mov(
            Operand::Reg(Reg::Rsp),
            Operand::Reg(Reg::Rbp),
        ));
        self.instructions.push(Instruction::Pop(Reg::Rbp));
        self.instructions.push(Instruction::Ret);

        peephole(&mut self.instructions);
    }

    fn asm(self) -> String {
        self.instructions
            .iter()
            .map(|instr| instr.to_string())
            .collect::<Vec<String>>()
            .join("\n")
    }
}
