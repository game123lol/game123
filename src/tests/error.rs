#[cfg(test)]
mod error {
    use crate::need_components;
    #[test]
    fn macro_err_correct() {
        let err = need_components!(TestSystem, TestComponent);
        assert_eq!(
            err.to_string(),
            "Can't run TestSystem without entity with TestComponent component"
        );
        let err = need_components!(TestSystem, TestComponent, TestComponent);
        assert_eq!(
            err.to_string(),
            "Can't run TestSystem without entity with TestComponent and TestComponent components"
        );
    }
}
