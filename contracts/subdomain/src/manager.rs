pub fn build_subdomain(label: &str, parent: &str) -> String {
    format!("{label}.{parent}")
}
