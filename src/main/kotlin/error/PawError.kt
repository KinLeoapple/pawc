package org.pawscript.error

/**
 * Base class for all PawScript errors.
 * @param message Human-readable message.
 * @param line Source line number (1-based).
 * @param col Source column number (1-based).
 * @param snippet Optional source snippet where error occurred.
 * @param hint Optional hint to resolve the error.
 */
sealed class PawError(
    override val message: String,
    open val line: Int,
    open val col: Int,
    open val snippet: String? = null,
    open val hint: String? = null
) : Throwable(message) {

    /** Error when a name is declared more than once */
    data class DuplicateDeclarationError(
        override val message: String,
        override val line: Int,
        override val col: Int,
        override val snippet: String? = null,
        override val hint: String? = null
    ) : PawError(message, line, col, snippet, hint)

    /** Lexical analysis error */
    data class LexError(
        override val message: String,
        override val line: Int,
        override val col: Int,
        override val snippet: String? = null,
        override val hint: String? = null
    ) : PawError(message, line, col, snippet, hint)

    /** Parsing / syntax error */
    data class ParseError(
        override val message: String,
        override val line: Int,
        override val col: Int,
        override val snippet: String? = null,
        override val hint: String? = null
    ) : PawError(message, line, col, snippet, hint)

    /** Type checking / semantic error */
    data class TypeError(
        override val message: String,
        override val line: Int,
        override val col: Int,
        override val snippet: String? = null,
        override val hint: String? = null
    ) : PawError(message, line, col, snippet, hint)

    /** Name resolution error (undefined or duplicate names) */
    data class NameError(
        override val message: String,
        override val line: Int,
        override val col: Int,
        override val snippet: String? = null,
        override val hint: String? = null
    ) : PawError(message, line, col, snippet, hint)

    /** Import or module resolution error */
    data class ImportError(
        override val message: String,
        override val line: Int,
        override val col: Int,
        override val snippet: String? = null,
        override val hint: String? = null
    ) : PawError(message, line, col, snippet, hint)

    /** Interface or record declaration error */
    data class DeclarationError(
        override val message: String,
        override val line: Int,
        override val col: Int,
        override val snippet: String? = null,
        override val hint: String? = null
    ) : PawError(message, line, col, snippet, hint)

    /** Function call related error */
    data class FunctionCallError(
        override val message: String,
        override val line: Int,
        override val col: Int,
        override val snippet: String? = null,
        override val hint: String? = null
    ) : PawError(message, line, col, snippet, hint)

    /** Field access or record instantiation error */
    data class FieldAccessError(
        override val message: String,
        override val line: Int,
        override val col: Int,
        override val snippet: String? = null,
        override val hint: String? = null
    ) : PawError(message, line, col, snippet, hint)

    /** Interface implementation validation error */
    data class InterfaceImplementationError(
        override val message: String,
        override val line: Int,
        override val col: Int,
        override val snippet: String? = null,
        override val hint: String? = null
    ) : PawError(message, line, col, snippet, hint)

    /** Type conversion ("as") error */
    data class TypeConversionError(
        override val message: String,
        override val line: Int,
        override val col: Int,
        override val snippet: String? = null,
        override val hint: String? = null
    ) : PawError(message, line, col, snippet, hint)

    /** Module dependency or import error */
    data class ModuleDependencyError(
        override val message: String,
        override val line: Int,
        override val col: Int,
        override val snippet: String? = null,
        override val hint: String? = null
    ) : PawError(message, line, col, snippet, hint)

    /** Control flow misuse error (e.g., return outside function) */
    data class ControlFlowError(
        override val message: String,
        override val line: Int,
        override val col: Int,
        override val snippet: String? = null,
        override val hint: String? = null
    ) : PawError(message, line, col, snippet, hint)

    /** Asynchronous programming related error (async/await misuse) */
    data class AsyncError(
        override val message: String,
        override val line: Int,
        override val col: Int,
        override val snippet: String? = null,
        override val hint: String? = null
    ) : PawError(message, line, col, snippet, hint)

    /** Runtime execution error */
    data class RuntimeError(
        override val message: String,
        override val line: Int,
        override val col: Int,
        override val snippet: String? = null,
        override val hint: String? = null
    ) : PawError(message, line, col, snippet, hint)

    override fun toString(): String {
        val loc = "[Line $line, Col $col]"
        val base = "${this::class.simpleName} $loc: $message"
        val snp = snippet?.let { "\n  at: $it" } ?: ""
        val hnt = hint?.let { "\n  hint: $it" } ?: ""
        return base + snp + hnt
    }
}
