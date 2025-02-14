//! Reina Smart Language (RSL)
//!
//! RSL is designed to be a safer, simpler smart contract language than Solidity,
//! with explicit type declarations and a Rust-like syntax. In Phase 1, we support
//! minimal contract, field, and function declarations. Future phases will add
//! concurrency, advanced validations, and integration with HPC tasks.

/// Error type for RSL parsing.
#[derive(Debug, PartialEq)]
pub enum RslError {
    /// Expected a specific token but found something else.
    Expected(String),
    /// General parse error with a message.
    ParseError(String),
}

/// Abstract Syntax Tree (AST) definitions for RSL.

/// Represents a contract with a name, fields, and functions.
#[derive(Debug, PartialEq)]
pub struct Contract {
    pub name: String,
    pub fields: Vec<Field>,
    pub functions: Vec<Function>,
}

/// Represents a field declaration, e.g., `let counter: u64;`
#[derive(Debug, PartialEq)]
pub struct Field {
    pub name: String,
    pub field_type: String,
}

/// Represents a function declaration.
#[derive(Debug, PartialEq)]
pub struct Function {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<String>,
    /// For Phase 1, we simply capture the function body as a string.
    pub body: String,
}

/// Represents a function parameter.
#[derive(Debug, PartialEq)]
pub struct Param {
    pub name: String,
    pub param_type: String,
}

/// Parses an RSL source string into a Contract AST.
///  
///  The expected syntax is:
///
///  contract ContractName {
///      let field_name: type;
///      fn function_name(param1: type1, param2: type2): return_type {
///          function body;
///      }
///      fn another_function() {
///          body;
///      }
///  } ```  Phase 1 only extracts the names and bodies.

pub fn parse_rsl(input: &str) -> Result<Contract, RslError> {
    let input = input.trim();
    // Expect the input to start with "contract"
    let rest = input.strip_prefix("contract")
        .ok_or_else(|| RslError::Expected("contract keyword".to_string()))?
        .trim();
    // Get contract name (token before first '{')
    let parts: Vec<&str> = rest.splitn(2, '{').collect();
    if parts.len() < 2 {
        return Err(RslError::Expected("{".to_string()));
    }
    let name = parts[0].trim().to_string();
    let body_str = parts[1].rsplitn(2, '}').nth(1)
        .ok_or_else(|| RslError::Expected("}".to_string()))?;
    let mut fields = Vec::new();
    let mut functions = Vec::new();
    // For simplicity, split the body by newlines.
    for line in body_str.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with("let ") {
            // Field: let field_name: type;
            let field_line = line.strip_prefix("let ")
                .ok_or_else(|| RslError::ParseError("Malformed field".to_string()))?;
            let parts: Vec<&str> = field_line.split(':').collect();
            if parts.len() != 2 {
                return Err(RslError::ParseError("Field missing ':'".to_string()));
            }
            let field_name = parts[0].trim().to_string();
            let field_type = parts[1].trim().trim_end_matches(';').to_string();
            fields.push(Field { name: field_name, field_type });
        } else if line.starts_with("fn ") {
            // Function: fn name(params) [: return_type] { body }
            // We'll extract until the first '{'
            let parts: Vec<&str> = line.splitn(2, '{').collect();
            if parts.len() != 2 {
                return Err(RslError::Expected("{".to_string()));
            }
            let header = parts[0].trim();
            let body = parts[1].trim().trim_end_matches('}').trim().to_string();
            // Remove "fn " prefix
            let header = header.strip_prefix("fn ")
                .ok_or_else(|| RslError::ParseError("Malformed function header".to_string()))?
                .trim();
            // Split header into signature and optional return type (split by ':' if present)
            let (sig, ret_type) = if let Some(pos) = header.find("):") {
                let sig_part = &header[..pos+1];
                let ret_part = header[pos+2..].trim();
                (sig_part, Some(ret_part.to_string()))
            } else {
                (header, None)
            };
            // sig should be like "function_name(param1: type, param2: type)"
            let sig_parts: Vec<&str> = sig.splitn(2, '(').collect();
            if sig_parts.len() != 2 {
                return Err(RslError::ParseError("Malformed function signature".to_string()));
            }
            let func_name = sig_parts[0].trim().to_string();
            let params_str = sig_parts[1].trim().trim_end_matches(')');
            let params: Vec<Param> = if params_str.is_empty() {
                Vec::new()
            } else {
                params_str.split(',')
                    .map(|p| {
                        let p_parts: Vec<&str> = p.split(':').collect();
                        if p_parts.len() != 2 {
                            return Err(RslError::ParseError("Malformed parameter".to_string()));
                        }
                        Ok(Param {
                            name: p_parts[0].trim().to_string(),
                            param_type: p_parts[1].trim().to_string(),
                        })
                    })
                    .collect::<Result<Vec<Param>, RslError>>()?
            };
            functions.push(Function { name: func_name, params, return_type: ret_type, body });
        }
    }
    Ok(Contract { name, fields, functions })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_sample_contract() {
        let sample = r#"
            contract MyContract {
                let counter: u64;
                fn increment(amount: u64) {
                    counter = counter + amount;
                }
                fn get_counter(): u64 {
                    return counter;
                }
            }
        "#;
        let ast = parse_rsl(sample).expect("Parsing failed");
        assert_eq!(ast.name, "MyContract");
        assert_eq!(ast.fields.len(), 1);
        assert_eq!(ast.functions.len(), 2);
        // Check first function has one parameter and no return type.
        let inc_fn = &ast.functions[0];
        assert_eq!(inc_fn.name, "increment");
        assert_eq!(inc_fn.params.len(), 1);
        assert!(inc_fn.return_type.is_none());
        // Check second function has a return type.
        let get_fn = &ast.functions[1];
        assert_eq!(get_fn.name, "get_counter");
        assert_eq!(get_fn.params.len(), 0);
        assert_eq!(get_fn.return_type, Some("u64".to_string()));
    }
}