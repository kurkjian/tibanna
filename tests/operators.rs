mod runner;

use runner::compile_and_run;

#[test]
fn test_add() {
    let prog = r#"
        let x = 12;
        let y = 34;
        exit(x + y);
    "#;

    let status = compile_and_run(prog);
    assert_eq!(status.code(), Some(46));
}

#[test]
fn test_sub() {
    let prog = r#"
        let x = 12;
        let y = 34;
        exit(y - x);
    "#;

    let status = compile_and_run(prog);
    assert_eq!(status.code(), Some(22));
}

#[test]
fn test_mul() {
    let prog = r#"
        let x = 6;
        let y = 9;
        exit(y * x);
    "#;

    let status = compile_and_run(prog);
    assert_eq!(status.code(), Some(54));
}
