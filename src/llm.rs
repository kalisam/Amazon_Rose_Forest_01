pub fn generate_code(original_code: &str) -> String {
    format!("{}\n// This code was improved by an LLM.", original_code)
}
