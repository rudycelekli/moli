use super::*;

#[test]
fn css_computed_font_serializes_resolved_longhands_and_retains_snapshots() {
    let mut vm = new_parsed_test_vm("https://computed-font.test/", "<!doctype html><body>");
    assert_eq!(
        vm.eval(include_str!("css_computed_font.js")).unwrap(),
        "true"
    );
}
