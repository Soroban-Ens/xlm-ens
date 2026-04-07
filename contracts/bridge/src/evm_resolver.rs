pub fn target_for_chain(chain: &str) -> Option<&'static str> {
    match chain {
        "base" => Some("0xbaseResolver"),
        "ethereum" => Some("0xethResolver"),
        "arbitrum" => Some("0xarbResolver"),
        _ => None,
    }
}
