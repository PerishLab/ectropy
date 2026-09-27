use super::seat::laws;

#[test]
fn pathed() {
    assert!(
        !laws("fn f() { let some_crate::Kind::Held { x } = y else { return; }; }")
            .contains(&"word".to_string())
    );
    assert!(laws("fn f() { let held_name = 1; }").contains(&"word".to_string()));
}
