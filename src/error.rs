use std::fmt;

/// 🐾 PawScript Error Type — cute but informative
#[derive(Debug, Clone)]
pub enum PawError {
    /// Syntax-level error, with line number
    Syntax {
        message: String,
    },

    /// Type mismatch or violation
    Type {
        message: String,
    },

    /// Variable is not defined
    UndefinedVariable {
        name: String,
    },

    /// Variable is defined twice in the same scope
    DuplicateDefinition {
        name: String,
    },

    Codegen {
        message: String,
    },

    /// Unexpected error (usually internal)
    Internal {
        message: String,
    },
}

impl fmt::Display for PawError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PawError::Syntax { message } => {
                writeln!(f, "❌  {}", message)?;
                writeln!(f, "👉 Double-check your code syntax!")
            }
            PawError::Type { message } => {
                writeln!(f, "❌  {}", message)?;
                writeln!(f, "👉 Make sure your types match properly!")
            }
            PawError::UndefinedVariable { name } => {
                writeln!(f, "❌  Undefined variable: '{}'", name)?;
                writeln!(f, "👉 Did you spell it correctly?")
            }
            PawError::DuplicateDefinition { name, } => {
                writeln!(f, "❌  The variable '{}' is already defined.", name)?;
                writeln!(f, "👉 Try using a different name!")
            }
            PawError::Codegen { message, .. } => {
                writeln!(f, "💥 Codegen error: {}", message)
            }
            PawError::Internal { message } => {
                writeln!(f, "💥 Internal error!")?;
                writeln!(f, "❌  {}", message)?;
                writeln!(f, "👉 Please report this to the PawScript maintainers.")
            }
        }
    }
}
