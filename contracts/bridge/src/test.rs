#[cfg(test)]
mod tests {
    use crate::{build_gmp_message, target_for_chain};

    #[test]
    fn builds_axelar_payloads() {
        let payload = build_gmp_message("timmy.xlm", "base", "0xbaseResolver").unwrap();
        assert!(payload.contains("timmy.xlm"));
        assert_eq!(target_for_chain("base"), Some("0xbaseResolver"));
    }
}
