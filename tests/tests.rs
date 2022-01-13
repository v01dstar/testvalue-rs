#[test]
fn test_value_adjust() {
    let f = || -> i32 {
        let mut var = 1;
        testvalue::adjust!("adjust_var", &mut var);
        var
    };
    assert_eq!(f(), 1);

    testvalue::set_callback("adjust_var", |var| {
        *var = 2;
    });
    assert_eq!(f(), 2);
}

#[test]
fn test_value_adjust_raii() {
    let f = || -> i32 {
        let mut var = 1;
        testvalue::adjust!("adjust_var1", &mut var);
        var
    };
    {
        let _raii = testvalue::ScopedCallback::new("adjust_var1", |var| {
            *var = 2;
        });
        assert_eq!(f(), 2);
    }

    assert_eq!(f(), 1);
}
