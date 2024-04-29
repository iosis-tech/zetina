#[macro_export]
macro_rules! hash {
    ($value:expr) => {{
        let mut hasher = DefaultHasher::new();
        $value.hash(&mut hasher);
        hasher.finish()
    }};
}
