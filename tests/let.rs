mod runner;

use runner::compile_and_run;

#[test]
fn test_it_works() {
    let prog = r#"
        let x = 69;
        exit(x);
    "#;

    let status = compile_and_run(prog);
    assert_eq!(status.code(), Some(69));
}

#[test]
fn test_let_with_ident() {
    let prog = r#"
        let x = 69;
        let y = x;
        exit(y);
    "#;

    let status = compile_and_run(prog);
    assert_eq!(status.code(), Some(69));
}

#[test]
fn test_let_with_bin_op() {
    let prog = r#"
        let x = 6 - 2;
        let y = x + 1;
        exit(y + x);
    "#;

    let status = compile_and_run(prog);
    assert_eq!(status.code(), Some(9));
}
