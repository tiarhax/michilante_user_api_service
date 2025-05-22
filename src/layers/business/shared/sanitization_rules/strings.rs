pub fn trim_both_sides(input: String) -> Result<String, String> {
    Ok(input.trim().to_string())
}


pub fn remove_double_spaces(input: String) -> Result<String, String> {
    let result = input.split_whitespace().collect::<Vec<&str>>().join(" ");
    Ok(result)
}

