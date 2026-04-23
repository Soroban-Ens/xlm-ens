use crate::constants::{
    MAX_NAME_LENGTH, MAX_REGISTRATION_YEARS, MIN_NAME_LENGTH, MIN_REGISTRATION_YEARS,
};
use crate::errors::CommonError;
use crate::types::Tld;

pub fn validate_label(label: &str) -> Result<(), CommonError> {
    let len = label.len();

    if len < MIN_NAME_LENGTH {
        return Err(CommonError::NameTooShort);
    }

    if len > MAX_NAME_LENGTH {
        return Err(CommonError::NameTooLong);
    }

    if !label
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
    {
        return Err(CommonError::InvalidCharacters);
    }

    if label.starts_with('-') || label.ends_with('-') {
        return Err(CommonError::InvalidLabelBoundary);
    }

    Ok(())
}

pub fn validate_owner(owner: &str) -> Result<(), CommonError> {
    if owner.trim().is_empty() {
        return Err(CommonError::EmptyOwner);
    }

    Ok(())
}

pub fn validate_registration_years(years: u64) -> Result<(), CommonError> {
    if !(MIN_REGISTRATION_YEARS..=MAX_REGISTRATION_YEARS).contains(&years) {
        return Err(CommonError::InvalidRegistrationPeriod);
    }

    Ok(())
}

pub fn parse_fqdn(name: &str) -> Result<(String, Tld), CommonError> {
    let mut parts = name.split('.');
    let label = parts.next().ok_or(CommonError::InvalidName)?;
    let tld = parts.next().ok_or(CommonError::MissingTld)?;

    if parts.next().is_some() {
        return Err(CommonError::InvalidName);
    }

    validate_label(label)?;
    let parsed_tld = Tld::parse(tld).ok_or(CommonError::UnsupportedTld)?;

    Ok((label.to_string(), parsed_tld))
}

pub fn validate_chain_name(chain: &str) -> Result<(), CommonError> {
    if chain.trim().is_empty() {
        return Err(CommonError::EmptyChainName);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{parse_fqdn, validate_chain_name, validate_label, validate_registration_years};
    use crate::constants::{
        MAX_NAME_LENGTH, MAX_REGISTRATION_YEARS, MIN_NAME_LENGTH, MIN_REGISTRATION_YEARS,
    };
    use crate::errors::CommonError;
    use crate::types::Tld;

    #[test]
    fn rejects_short_labels() {
        let short_label = "a".repeat(MIN_NAME_LENGTH - 1);

        assert_eq!(validate_label(&short_label), Err(CommonError::NameTooShort));
    }

    #[test]
    fn rejects_long_labels() {
        let long_label = "a".repeat(MAX_NAME_LENGTH + 1);

        assert_eq!(validate_label(&long_label), Err(CommonError::NameTooLong));
    }

    #[test]
    fn rejects_unsupported_tlds() {
        assert_eq!(parse_fqdn("valid.eth"), Err(CommonError::UnsupportedTld));
    }

    #[test]
    fn rejects_invalid_characters() {
        assert_eq!(validate_label("ab_"), Err(CommonError::InvalidCharacters));
        assert_eq!(parse_fqdn("ab_.xlm"), Err(CommonError::InvalidCharacters));
    }

    #[test]
    fn rejects_labels_with_invalid_hyphen_boundaries() {
        assert_eq!(
            validate_label("-abc"),
            Err(CommonError::InvalidLabelBoundary)
        );
        assert_eq!(
            validate_label("abc-"),
            Err(CommonError::InvalidLabelBoundary)
        );
    }

    #[test]
    fn accepts_valid_fqdn() {
        assert_eq!(
            parse_fqdn("abc-123.xlm"),
            Ok(("abc-123".to_string(), Tld::Xlm))
        );
    }

    #[test]
    fn enforces_registration_year_bounds() {
        assert_eq!(validate_registration_years(MIN_REGISTRATION_YEARS), Ok(()));
        assert_eq!(validate_registration_years(MAX_REGISTRATION_YEARS), Ok(()));
        assert_eq!(
            validate_registration_years(MIN_REGISTRATION_YEARS - 1),
            Err(CommonError::InvalidRegistrationPeriod)
        );
        assert_eq!(
            validate_registration_years(MAX_REGISTRATION_YEARS + 1),
            Err(CommonError::InvalidRegistrationPeriod)
        );
    }

    #[test]
    fn validates_chain_name_presence() {
        assert_eq!(validate_chain_name("stellar"), Ok(()));
        assert_eq!(validate_chain_name("   "), Err(CommonError::EmptyChainName));
    }
}
