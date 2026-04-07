use xlm_ns_common::validation::{parse_fqdn, validate_label};
use xlm_ns_common::{CommonError, Tld};

pub fn build_subdomain(label: &str, parent: &str) -> Result<String, CommonError> {
    validate_label(label)?;
    let (parent_label, tld) = parse_fqdn(parent)?;

    Ok(format!("{label}.{parent_label}.{}", tld.as_str()))
}

pub fn split_subdomain(name: &str) -> Result<(String, String, Tld), CommonError> {
    let mut parts = name.split('.');
    let child = parts.next().ok_or(CommonError::InvalidName)?;
    let parent = parts.next().ok_or(CommonError::InvalidName)?;
    let tld = parts.next().ok_or(CommonError::MissingTld)?;
    if parts.next().is_some() {
        return Err(CommonError::InvalidName);
    }

    validate_label(child)?;
    validate_label(parent)?;
    let tld = Tld::parse(tld).ok_or(CommonError::UnsupportedTld)?;
    Ok((child.to_string(), parent.to_string(), tld))
}
