use crate::layers::business::shared::business_rules::FieldValidationResult;
pub fn non_empty(input: &str, field_name: &str, message: String) -> FieldValidationResult {
    if input.is_empty() {
        FieldValidationResult::Valid
    } else {
        FieldValidationResult::Invalid(field_name.to_string(), message)
    }
}