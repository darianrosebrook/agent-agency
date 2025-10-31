//! Test fixture for refactor scenario
//!
//! Provides a Rust module with code smells that need refactoring:
//! - Complex functions (>100 lines)
//! - Deep nesting
//! - Naming violations
//! - Missing documentation

use std::collections::HashMap;

/// Complex function with multiple responsibilities - needs refactoring
pub fn process_user_data(input: &str) -> Result<HashMap<String, String>, String> {
    // Input validation (should be extracted)
    if input.is_empty() {
        return Err("Input cannot be empty".to_string());
    }

    if input.len() > 1000 {
        return Err("Input too long".to_string());
    }

    // Parsing logic (should be extracted)
    let mut result = HashMap::new();
    let parts: Vec<&str> = input.split(',').collect();

    if parts.len() < 2 {
        return Err("Invalid input format".to_string());
    }

    // Complex nested logic for processing user info
    for (index, part) in parts.iter().enumerate() {
        let trimmed = part.trim();

        if index == 0 {
            // Process name
            if trimmed.is_empty() {
                return Err("Name cannot be empty".to_string());
            }

            if trimmed.len() < 2 {
                return Err("Name too short".to_string());
            }

            if trimmed.chars().any(|c| !c.is_alphabetic() && !c.is_whitespace()) {
                return Err("Name contains invalid characters".to_string());
            }

            result.insert("name".to_string(), trimmed.to_string());
        } else if index == 1 {
            // Process age
            match trimmed.parse::<u32>() {
                Ok(age) => {
                    if age < 0 {
                        return Err("Age cannot be negative".to_string());
                    }

                    if age > 150 {
                        return Err("Age too high".to_string());
                    }

                    result.insert("age".to_string(), age.to_string());
                }
                Err(_) => {
                    return Err("Invalid age format".to_string());
                }
            }
        } else if index == 2 {
            // Process email
            if !trimmed.contains('@') {
                return Err("Invalid email format".to_string());
            }

            let email_parts: Vec<&str> = trimmed.split('@').collect();
            if email_parts.len() != 2 {
                return Err("Invalid email format".to_string());
            }

            let domain = email_parts[1];
            if !domain.contains('.') {
                return Err("Invalid email domain".to_string());
            }

            result.insert("email".to_string(), trimmed.to_string());
        } else {
            // Additional fields
            result.insert(format!("field_{}", index), trimmed.to_string());
        }
    }

    // Validation logic (should be extracted)
    if let Some(name) = result.get("name") {
        if name.len() > 50 {
            return Err("Name too long".to_string());
        }
    }

    if let Some(email) = result.get("email") {
        if email.len() > 100 {
            return Err("Email too long".to_string());
        }
    }

    Ok(result)
}

/// Another complex function with multiple levels of nesting
pub fn calculate_user_score(user_data: &HashMap<String, String>) -> Result<f64, String> {
    let mut score = 0.0;

    // Nested validation and scoring logic
    if let Some(name) = user_data.get("name") {
        if !name.is_empty() {
            let name_length = name.len() as f64;
            score += name_length * 0.1;

            if name_length > 10.0 {
                if name.chars().all(|c| c.is_alphabetic() || c.is_whitespace()) {
                    score += 5.0;

                    if name.contains(' ') {
                        score += 2.0;

                        if name.split_whitespace().count() >= 2 {
                            score += 3.0;
                        }
                    }
                }
            }
        } else {
            return Err("Name is required for scoring".to_string());
        }
    }

    // Age scoring with deep nesting
    if let Some(age_str) = user_data.get("age") {
        if let Ok(age) = age_str.parse::<u32>() {
            if age > 0 {
                if age < 18 {
                    score += 1.0;
                } else if age < 30 {
                    score += 3.0;

                    if age >= 25 {
                        score += 1.0;
                    }
                } else if age < 50 {
                    score += 4.0;

                    if age >= 40 {
                        score += 1.0;
                    }
                } else {
                    score += 2.0;
                }
            }
        } else {
            return Err("Invalid age format".to_string());
        }
    }

    // Email scoring
    if let Some(email) = user_data.get("email") {
        if email.contains('@') {
            score += 2.0;

            if email.ends_with(".com") || email.ends_with(".org") {
                score += 1.0;
            }
        }
    }

    Ok(score)
}

/// Function with poor naming and unclear purpose
pub fn do_stuff_with_data(data: Vec<HashMap<String, String>>) -> Vec<Result<f64, String>> {
    data.into_iter().map(|item| {
        match process_user_data(&serde_json::to_string(&item).unwrap_or_default()) {
            Ok(processed) => calculate_user_score(&processed),
            Err(e) => Err(e),
        }
    }).collect()
}

/// Test module for the refactor target
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_user_data_valid_input() {
        let input = "John Doe, 25, john@example.com";
        let result = process_user_data(input).unwrap();

        assert_eq!(result.get("name").unwrap(), "John Doe");
        assert_eq!(result.get("age").unwrap(), "25");
        assert_eq!(result.get("email").unwrap(), "john@example.com");
    }

    #[test]
    fn test_process_user_data_empty_input() {
        let result = process_user_data("");
        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_user_score() {
        let mut data = HashMap::new();
        data.insert("name".to_string(), "John Doe".to_string());
        data.insert("age".to_string(), "30".to_string());
        data.insert("email".to_string(), "john@example.com".to_string());

        let score = calculate_user_score(&data).unwrap();
        assert!(score > 0.0);
    }
}



