#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmTarget {
    pub chain: &'static str,
    pub resolver: &'static str,
    pub gateway: &'static str,
}

pub fn target_for_chain(chain: &str) -> Option<EvmTarget> {
    match chain {
        "base" => Some(EvmTarget {
            chain: "base",
            resolver: "0xbaseResolver",
            gateway: "0xbaseGateway",
        }),
        "ethereum" => Some(EvmTarget {
            chain: "ethereum",
            resolver: "0xethResolver",
            gateway: "0xethGateway",
        }),
        "arbitrum" => Some(EvmTarget {
            chain: "arbitrum",
            resolver: "0xarbResolver",
            gateway: "0xarbGateway",
        }),
        _ => None,
    }
}
