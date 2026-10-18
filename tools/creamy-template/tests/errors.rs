use creamy_template::{Id, error::TemplateError};

#[test]
fn test_id_empty_input() {
    let result = Id::new("");
    assert!(matches!(result, Err(TemplateError::EmptyInput)));
}

#[test]
fn test_id_invalid_format_too_many_segments() {
    // "org.project.module.extra" has 4 segments
    let result = Id::new("org.project.module.extra");
    assert!(matches!(result, Err(TemplateError::InvalidIdFormat)));
}

#[test]
fn test_id_invalid_format_too_few_segments() {
    // "org.project" has 2 segments
    let result = Id::new("org.project");
    assert!(matches!(result, Err(TemplateError::InvalidIdFormat)));
}

#[test]
fn test_id_invalid_segment_start_char() {
    // "123.project.module" starts with a number
    let result = Id::new("123.project.module");
    assert!(matches!(result, Err(TemplateError::InvalidIdFormat)));
}

#[test]
fn test_id_invalid_segment_chars() {
    // "org.proj-ect.module" contains a hyphen
    let result = Id::new("org.proj-ect.module");
    assert!(matches!(result, Err(TemplateError::InvalidIdFormat)));
}

#[test]
fn test_generate_template_invalid_id() {
    // This test ensures that even if we try to construct a Template with an invalid ID,
    // the Id::new() check prevents it.
    let result = Id::new("invalid_id");
    assert!(result.is_err());
}
