// File: src/error/error.rs

use std::fmt;

/// 🐾 PawScript Error Type — informative and spanned
#[derive(Debug, Clone)]
pub enum PawError {
    /// Syntax error with span and optional hint
    Syntax {
        code: &'static str,
        message: String,
        line: usize,
        column: usize,
        snippet: Option<String>,
        hint: Option<String>,
    },

    /// Type error with span and optional hint
    Type {
        code: &'static str,
        message: String,
        line: usize,
        column: usize,
        snippet: Option<String>,
        hint: Option<String>,
    },

    /// Undefined variable error
    UndefinedVariable {
        code: &'static str,
        name: String,
        line: usize,
        column: usize,
        snippet: Option<String>,
        hint: Option<String>,
    },

    /// Duplicate definition error
    DuplicateDefinition {
        code: &'static str,
        name: String,
        line: usize,
        column: usize,
        snippet: Option<String>,
        hint: Option<String>,
    },

    /// Code generation/runtime error
    Codegen {
        code: &'static str,
        message: String,
        line: usize,
        column: usize,
        snippet: Option<String>,
        hint: Option<String>,
    },

    /// Internal error
    Internal {
        code: &'static str,
        message: String,
        line: usize,
        column: usize,
        snippet: Option<String>,
        hint: Option<String>,
    },
}

impl fmt::Display for PawError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PawError::Syntax { code, message, line, column, snippet, hint } => {
                writeln!(f, "[{}] Syntax Error at {}:{}: {}", code, line, column, message)?;
                if let Some(ref src) = snippet {
                    writeln!(f, "    {}", src)?;
                }
                if let Some(ref h) = hint {
                    writeln!(f, "Hint: {}", h)?;
                }
                Ok(())
            }

            PawError::Type { code, message, line, column, snippet, hint } => {
                writeln!(f, "[{}] Type Error at {}:{}: {}", code, line, column, message)?;
                if let Some(ref src) = snippet {
                    writeln!(f, "    {}", src)?;
                }
                if let Some(ref h) = hint {
                    writeln!(f, "Hint: {}", h)?;
                }
                Ok(())
            }

            PawError::UndefinedVariable { code, name, line, column, snippet, hint } => {
                writeln!(f, "[{}] Undefined Variable '{}' at {}:{}", code, name, line, column)?;
                if let Some(ref src) = snippet {
                    writeln!(f, "    {}", src)?;
                }
                if let Some(ref h) = hint {
                    writeln!(f, "Hint: {}", h)?;
                }
                Ok(())
            }

            PawError::DuplicateDefinition { code, name, line, column, snippet, hint } => {
                writeln!(f, "[{}] Duplicate definition '{}' at {}:{}", code, name, line, column)?;
                if let Some(ref src) = snippet {
                    writeln!(f, "    {}", src)?;
                }
                if let Some(ref h) = hint {
                    writeln!(f, "Hint: {}", h)?;
                }
                Ok(())
            }

            PawError::Codegen { code, message, line, column, snippet, hint } => {
                writeln!(f, "[{}] Codegen Error at {}:{}: {}", code, line, column, message)?;
                if let Some(ref src) = snippet {
                    writeln!(f, "    {}", src)?;
                }
                if let Some(ref h) = hint {
                    writeln!(f, "Hint: {}", h)?;
                }
                Ok(())
            }

            PawError::Internal { code, message, line, column, snippet: _snippet, hint } => {
                writeln!(f, "[{}] Internal Error at {}:{}", code, line, column)?;
                writeln!(f, "    {}", message)?;
                if let Some(ref h) = hint {
                    writeln!(f, "Hint: {}", h)?;
                }
                Ok(())
            }
        }
    }
}
