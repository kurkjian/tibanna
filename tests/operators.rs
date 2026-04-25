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

#[test]
fn test_triple_add() {
    let prog = r#"
        let x = 12;
        let y = 34;
        let z = 1;
        exit(x + y + z);
    "#;

    let status = compile_and_run(prog);
    assert_eq!(status.code(), Some(47));
}

#[test]
fn test_mul_sub_add() {
    let prog = r#"
        let x = 3;
        let y = 5;
        let z = 7;
        let w = 11;
        exit(x * y - z + w);
    "#;

    let status = compile_and_run(prog);
    assert_eq!(status.code(), Some(19));
}

#[test]
fn test_mul_precedence() {
    let prog = r#"
        let x = 3;
        let y = 5;
        let z = 1;
        let w = 3;
        exit(x + y - z * w);
    "#;

    let status = compile_and_run(prog);
    assert_eq!(status.code(), Some(5));
}
