use validator::ValidationError;

const VALID_PERMISSIONS: &[&str] = &["admin", "token"];

pub fn validate_permissions(permissions: &[String]) -> Result<(), ValidationError> {
    if permissions.iter().all(|p| VALID_PERMISSIONS.contains(&p.as_str())) {
        Ok(())
    } else {
        Err(ValidationError::new("invalid_permission"))
    }
}
