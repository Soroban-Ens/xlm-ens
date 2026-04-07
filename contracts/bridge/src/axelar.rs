use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct AxelarMessage<'a> {
    pub name: &'a str,
    pub destination_chain: &'a str,
    pub resolver: &'a str,
}

pub fn build_gmp_message(
    name: &str,
    destination_chain: &str,
    resolver: &str,
) -> serde_json::Result<String> {
    serde_json::to_string(&AxelarMessage {
        name,
        destination_chain,
        resolver,
    })
}
