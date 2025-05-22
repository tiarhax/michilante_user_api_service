
pub mod strings;
pub fn pipe_all<DType>(rules: Vec<impl Fn(DType) -> Result<DType, String>>, input: &DType) -> Result<DType, String>
where DType: Clone
{
    if rules.is_empty() {
        return Ok(input.clone());
    }
    let mut current_value = input.clone();
    for rule in rules {
        current_value = rule(input.clone()).map_err(|e| format!("Sanitization rule error: {}", e))?;
    }

    Ok(current_value)
}
