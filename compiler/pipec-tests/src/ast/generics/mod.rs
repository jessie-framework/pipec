#[test]
fn test_generics() {
    {
        crate::test_file_generation!("typegenerics.pipec");
    }
    {
        crate::test_file_generation!("functiongenerics.pipec");
    }
}
