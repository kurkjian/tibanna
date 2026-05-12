use std::process::{Command, ExitStatus};
use tempfile::TempDir;
use tibanna::{
    backend::{
        constant::ConstantFolding, dce::DeadCodeElimination, driver::Driver,
        targets::x86_64::X86_64,
    },
    ir::lower::lower_program,
    lexer::Lexer,
    parser::Parser,
    resolver::Resolver,
};

pub fn compile_and_run(prog: &str) -> ExitStatus {
    let dir = TempDir::new().unwrap();
    let asm = dir.path().join("prog.asm");
    let obj = dir.path().join("prog.o");
    let exe = dir.path().join("prog");

    let prog = Parser::new(Lexer::new(prog).tokenize().unwrap())
        .parse()
        .unwrap();
    let resolved = Resolver::default().resolve_program(prog).unwrap();
    let ir = lower_program(resolved);
    let backend = Driver::new(X86_64::default(), asm.clone())
        .with_pass(ConstantFolding)
        .with_pass(DeadCodeElimination);

    backend.run(ir);

    let _nasm = Command::new("nasm")
        .arg("-f")
        .arg("elf64")
        .arg("-o")
        .arg(&obj)
        .arg(&asm)
        .output()
        .unwrap();

    let _ld = Command::new("ld")
        .arg("-o")
        .arg(&exe)
        .arg(&obj)
        .output()
        .unwrap();

    Command::new(&exe).status().expect("failed to run program")
}
