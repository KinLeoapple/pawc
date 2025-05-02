// src/error/error.rs

use colored::Colorize;
use std::fmt;

/// 🐾 PawScript Error Type — cute but informative and spanned
#[derive(Debug, Clone)]
pub enum PawError {
    /// Syntax error with span and optional hint
    Syntax {
        file: String,
        code: &'static str,
        message: String,
        line: usize,
        column: usize,
        snippet: Option<String>,
        hint: Option<String>,
    },

    /// Type error with span and optional hint
    Type {
        file: String,
        code: &'static str,
        message: String,
        line: usize,
        column: usize,
        snippet: Option<String>,
        hint: Option<String>,
    },

    /// Undefined variable error
    UndefinedVariable {
        file: String,
        code: &'static str,
        name: String,
        line: usize,
        column: usize,
        snippet: Option<String>,
        hint: Option<String>,
    },

    /// Duplicate definition error
    DuplicateDefinition {
        file: String,
        code: &'static str,
        name: String,
        line: usize,
        column: usize,
        snippet: Option<String>,
        hint: Option<String>,
    },

    /// Code generation/runtime error
    Codegen {
        file: String,
        code: &'static str,
        message: String,
        line: usize,
        column: usize,
        snippet: Option<String>,
        hint: Option<String>,
    },

    /// Internal error
    Internal {
        file: String,
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
            PawError::Syntax {
                file,
                code,
                message,
                line,
                column,
                snippet,
                hint,
            } => {
                let file_hint = format!("{}:{}:{}", file, line, column);
                writeln!(f, "🐾 [{}] Syntax Error in {} 🐾", code, file_hint.yellow().underline())?;
                writeln!(f, "   💬 {}", message)?;
                if let Some(src) = snippet {
                    writeln!(f, "   📜  {}", src)?;
                }
                if let Some(h) = hint {
                    writeln!(f, "   💡 Hint: {}", h)?;
                }
                Ok(())
            }

            PawError::Type {
                file,
                code,
                message,
                line,
                column,
                snippet,
                hint,
            } => {
                let file_hint = format!("{}:{}:{}", file, line, column);
                writeln!(
                    f,
                    "🐾 [{}] Type Error in {} 🐾",
                    code,
                    file_hint.yellow().underline()
                )?;
                writeln!(f, "   💬 {}", message)?;
                if let Some(src) = snippet {
                    writeln!(f, "   📜 {}", src)?;
                }
                if let Some(h) = hint {
                    writeln!(f, "   💡 Hint: {}", h)?;
                }
                Ok(())
            }

            PawError::UndefinedVariable {
                file,
                code,
                name,
                line,
                column,
                snippet,
                hint,
            } => {
                let file_hint = format!("{}:{}:{}", file, line, column);
                writeln!(
                    f,
                    "🐾 [{}] Oops! Undefined variable '{}' in {} 🐾",
                    code,
                    name,
                    file_hint.yellow()
                )?;
                if let Some(src) = snippet {
                    writeln!(f, "   📜 {}", src)?;
                }
                if let Some(h) = hint {
                    writeln!(f, "   💡 Hint: {}", h)?;
                }
                Ok(())
            }

            PawError::DuplicateDefinition {
                file,
                code,
                name,
                line,
                column,
                snippet,
                hint,
            } => {
                let file_hint = format!("{}:{}:{}", file, line, column);
                writeln!(
                    f,
                    "🐾 [{}] Duplicate definition '{}' in {} 🐾",
                    code,
                    name,
                    file_hint.yellow().underline()
                )?;
                if let Some(src) = snippet {
                    writeln!(f, "   📜 {}", src)?;
                }
                if let Some(h) = hint {
                    writeln!(f, "   💡 Hint: {}", h)?;
                }
                Ok(())
            }

            PawError::Codegen {
                file,
                code,
                message,
                line,
                column,
                snippet,
                hint,
            } => {
                writeln!(
                    f,
                    "🐾 [{}] Codegen Error in {} at {} 🐾",
                    code,
                    file.yellow().underline(),
                    format!("{}:{}", line, column).yellow().underline()
                )?;
                writeln!(f, "   💥 {}", message)?;
                if let Some(src) = snippet {
                    writeln!(f, "   📜 {}", src)?;
                }
                if let Some(h) = hint {
                    writeln!(f, "   💡 Hint: {}", h)?;
                }
                Ok(())
            }

            PawError::Internal {
                file,
                code,
                message,
                line,
                column,
                snippet: _snippet,
                hint,
            } => {
                let file_hint = format!("{}:{}:{}", file, line, column);
                writeln!(
                    f,
                    "🐾 [{}] Internal Error in {} 🐾",
                    code,
                    file_hint.yellow().underline()
                )?;
                writeln!(f, "   💥 {}", message)?;
                if let Some(h) = hint {
                    writeln!(f, "   💡 Hint: {}", h)?;
                }
                Ok(())
            }
        }
    }
}
