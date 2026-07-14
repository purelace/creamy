use std::{env, fs, path::PathBuf};

/// Checks if a path meets all requirements:
/// 1. Is a directory
/// 2. Supports absolute and relative paths
/// 3. Supports environment variables ($`ENV_VAR` syntax)
/// 4. Linux only
///
/// # Arguments
/// * `path` - The path to check (can include environment variables)
///
/// # Returns
/// * `Ok(PathBuf)` - The resolved, canonicalized path if valid
/// * `Err(String)` - Error message explaining what went wrong
pub fn to_absolute_path(path: &str) -> Result<PathBuf, String> {
    // Step 1: Expand environment variables
    //let path_str = path
    //    .as_ref()
    //    .to_str()
    //    .ok_or("Invalid path: contains non-UTF8 characters")?;

    let expanded = expand_env_vars(path)
        .map_err(|e| format!("Failed to expand environment variables: {e}"))?;

    let expanded_path = PathBuf::from(expanded);

    // Step 2: Resolve to absolute path (handles relative paths)
    let absolute_path = if expanded_path.is_absolute() {
        expanded_path
    } else {
        // Relative to current working directory
        env::current_dir()
            .map_err(|e| format!("Failed to get current working directory: {e}"))?
            .join(expanded_path)
    };

    // Step 3: Canonicalize to resolve symlinks and get clean path
    let canonicalized = fs::canonicalize(&absolute_path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            format!("Path does not exist: {}", absolute_path.display())
        } else {
            format!("Failed to canonicalize path: {e}")
        }
    })?;

    // Step 4: Verify it's a directory
    if !canonicalized.is_dir() {
        return Err(format!(
            "Path is not a directory: {}",
            canonicalized.display()
        ));
    }

    Ok(canonicalized)
}

/// Expands environment variables in a string using $VAR syntax
///
/// Supports:
/// - $`VAR_NAME`
/// - ${`VAR_NAME`}
fn expand_env_vars(input: &str) -> Result<String, String> {
    let mut result = String::new();
    let mut chars = input.chars().peekable();
    let mut in_var = false;
    let mut var_name = String::new();
    let mut in_brace = false;

    while let Some(c) = chars.next() {
        match c {
            '$' => {
                in_var = true;
                var_name.clear();
                in_brace = false;

                // Check if it's ${VAR}
                if let Some(&'{') = chars.peek() {
                    chars.next(); // consume '{'
                    in_brace = true;
                }
            }
            c if in_var => {
                if c.is_ascii_alphanumeric() || c == '_' {
                    var_name.push(c);
                } else if in_brace && c == '}' {
                    // End of ${VAR}
                    let value = env::var(&var_name)
                        .map_err(|_| format!("Environment variable '{var_name}' not found"))?;
                    result.push_str(&value);
                    in_var = false;
                    var_name.clear();
                    in_brace = false;
                } else {
                    // End of $VAR
                    let value = env::var(&var_name)
                        .map_err(|_| format!("Environment variable '{var_name}' not found"))?;
                    result.push_str(&value);
                    result.push(c);
                    in_var = false;
                    var_name.clear();
                    in_brace = false;
                }
            }
            c => {
                result.push(c);
            }
        }
    }

    // Handle case where string ends with a variable
    if in_var && !var_name.is_empty() {
        let value = env::var(&var_name)
            .map_err(|_| format!("Environment variable '{var_name}' not found"))?;
        result.push_str(&value);
    }

    Ok(result)
}
