mod runner;

use runner::compile_and_run;

#[test]
fn test_single_line() {
    let prog = r#"
        let x = 69;
        // comment
        exit(x);
    "#;

    let status = compile_and_run(prog);
    assert_eq!(status.code(), Some(69));
}

#[test]
fn test_inline() {
    let prog = r#"
        let x = 69; // comment
        exit(x);//no whitespace
    "#;

    let status = compile_and_run(prog);
    assert_eq!(status.code(), Some(69));
}
