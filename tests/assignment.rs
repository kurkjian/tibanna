mod runner;

use runner::compile_and_run;

#[test]
fn test_it_works() {
    let prog = r#"
        let x = 12;
        x = 13;
        exit(x);
    "#;

    let status = compile_and_run(prog);
    assert_eq!(status.code(), Some(13));
}

#[test]
fn test_assignment_to_ident() {
    let prog = r#"
        let x = 12;
        let y = 13;
        x = y;
        exit(x);
    "#;

    let status = compile_and_run(prog);
    assert_eq!(status.code(), Some(13));
}

#[test]
fn test_assignment_with_expr() {
    let prog = r#"
        let x = 12;
        let y = 13;
        x = y * 2;
        x = x - 1;
        exit(x);
    "#;

    let status = compile_and_run(prog);
    assert_eq!(status.code(), Some(25));
}
