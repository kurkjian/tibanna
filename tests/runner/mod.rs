use std::process::{Command, ExitStatus};
use tempfile::TempDir;
use tibanna::{compile::Compiler, lexer::Lexer, parser::Parser};

pub fn compile_and_run(prog: &str) -> ExitStatus {
    let instrs = Compiler::new().compile(Parser::new(Lexer::new(prog).tokenize().unwrap()).parse());
    let asm_instrs = instrs
        .into_iter()
        .fold(String::new(), |acc, instr| format!("{}\n{}", acc, instr));

    let as_str = r#"
            global _start
            _start:
            push rbp
            mov rbp, rsp
        "#
    .to_string()
        + &asm_instrs;

    let dir = TempDir::new().unwrap();
    let asm = dir.path().join("prog.asm");
    let obj = dir.path().join("prog.o");
    let exe = dir.path().join("prog");
    std::fs::write(&asm, as_str.as_bytes()).unwrap();

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
