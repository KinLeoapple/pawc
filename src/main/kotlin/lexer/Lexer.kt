package org.pawscript.lexer

import org.pawscript.error.PawError.LexError


class Lexer(private val source: String) {
    private val tokens = mutableListOf<Token>()
    private var start = 0
    private var current = 0
    private var line = 1
    private var col = 1
    private var inTemplate = false

    private val keywords = mapOf(
        "say" to TokenKind.SAY,
        "ask" to TokenKind.ASK,
        "let" to TokenKind.LET,
        "fun" to TokenKind.FUN,
        "if" to TokenKind.IF,
        "else" to TokenKind.ELSE,
        "loop" to TokenKind.LOOP,
        "in" to TokenKind.IN,
        "return" to TokenKind.RETURN,
        "bark" to TokenKind.BARK,
        "async" to TokenKind.ASYNC,
        "await" to TokenKind.AWAIT,
        "import" to TokenKind.IMPORT,
        "as"     to TokenKind.AS,
        "record" to TokenKind.RECORD,
        "tail" to TokenKind.TAIL,
        "sniff" to TokenKind.SNIFF,
        "snatch" to TokenKind.SNATCH,
        "lastly" to TokenKind.LASTLY,
        "break"    to TokenKind.BREAK,
        "continue" to TokenKind.CONTINUE,
        "nopaw" to TokenKind.NOPAW
    )

    /**
     * Tokenize the source into a list of tokens or throw LexError on failure.
     */
    @Throws(LexError::class)
    fun tokenize(): List<Token> {
        while (!isAtEnd()) {
            start = current
            scanToken()
        }
        tokens.add(Token(TokenKind.EOF, "", line, col))
        return tokens
    }

    private fun scanToken() {
        val c = advance()
        when {
            c == '\n' -> handleNewLine()
            c.isWhitespace() -> return
            c == '#' -> skipComment()
            inTemplate && c == '}' -> addToken(TokenKind.TEMPLATE_EXPR_END, "}")
            c == '`' -> templateStart()
            c == '"' -> stringLiteral()
            c == '\'' -> charLiteral()
            c.isLetter() || c == '_' -> identifierOrKeyword()
            c.isDigit() -> number()
            else -> symbolOrError(c)
        }
    }

    private fun handleNewLine() {
        line++
        col = 1
    }

    private fun skipComment() {
        while (peek() != '\n' && !isAtEnd()) advance()
    }

    private fun identifierOrKeyword() {
        while (peek().isLetterOrDigit() || peek() == '_') advance()
        emitKeywordOrIdentifier()
    }

    private fun emitKeywordOrIdentifier() {
        val text = source.substring(start, current)
        val kind = keywords[text] ?: TokenKind.IDENTIFIER
        addToken(kind, text)
    }

    private fun number() {
        while (peek().isDigit()) advance()
        if (peek() == '.' && peekNext().isDigit()) {
            advance()
            while (peek().isDigit()) advance()
            addToken(TokenKind.FLOAT_LITERAL, source.substring(start, current))
        } else {
            addToken(TokenKind.INT_LITERAL, source.substring(start, current))
        }
    }

    private fun stringLiteral() {
        val sb = StringBuilder()
        while (!isAtEnd() && peek() != '"') {
            if (peek() == '\\') {
                advance()
                sb.append(parseEscape())
            } else {
                sb.append(advance())
            }
        }
        if (isAtEnd()) {
            throw LexError("Unterminated string literal", line, col)
        }
        advance() // closing '"'
        addToken(TokenKind.STRING_LITERAL, sb.toString())
    }

    private fun charLiteral() {
        if (isAtEnd()) throw LexError("Unterminated char literal", line, col)
        val ch = if (peek() == '\\') {
            advance(); parseEscape()
        } else advance()
        if (peek() != '\'') throw LexError("Unterminated char literal", line, col)
        advance() // closing '\''
        addToken(TokenKind.CHAR_LITERAL, ch.toString())
    }

    private fun parseEscape(): Char {
        if (isAtEnd()) throw LexError("Invalid escape sequence at end of input", line, col)
        val escape = advance()
        return when (escape) {
            'n'  -> '\n'
            'r'  -> '\r'
            't'  -> '\t'
            '\\' -> '\\'
            '"' -> '"'
            '\''-> '\''
            '`'  -> '`'
            '$'  -> '$'
            else -> throw LexError("Invalid escape sequence: \\$escape", line, col)
        }
    }

    private fun templateStart() {
        inTemplate = true
        val sb = StringBuilder()
        while (!isAtEnd()) {
            when {
                peek() == '`' -> {
                    advance()
                    addToken(TokenKind.TEMPLATE_TEXT, sb.toString())
                    inTemplate = false
                    return
                }
                peek() == '$' && peekNext() == '{' -> {
                    addToken(TokenKind.TEMPLATE_TEXT, sb.toString())
                    sb.clear()
                    advance(); advance()
                    addToken(TokenKind.TEMPLATE_EXPR_START, "\${")
                    inTemplate = true
                    return
                }
                peek() == '\\' -> {
                    advance(); sb.append(parseEscape())
                }
                else -> sb.append(advance())
            }
        }
        throw LexError("Unterminated template literal", line, col)
    }

    private fun symbolOrError(c: Char) {
        when (c) {
            '+' -> addToken(TokenKind.PLUS)
            '-' -> if (match('>')) addToken(TokenKind.ARROW) else addToken(TokenKind.MINUS)
            '*' -> addToken(TokenKind.STAR)
            '/' -> addToken(TokenKind.SLASH)
            '%' -> addToken(TokenKind.PERCENT)
            '?' -> addToken(TokenKind.QUESTION)
            '(' -> addToken(TokenKind.LPAREN)
            ')' -> addToken(TokenKind.RPAREN)
            '{' -> addToken(TokenKind.LBRACE)
            '}' -> addToken(TokenKind.RBRACE)
            '[' -> addToken(TokenKind.LBRACKET)
            ']' -> addToken(TokenKind.RBRACKET)
            ',' -> addToken(TokenKind.COMMA)
            '.' -> if (match('.')) addToken(TokenKind.RANGE) else addToken(TokenKind.DOT)
            ':' -> addToken(TokenKind.COLON)
            ';' -> addToken(TokenKind.SEMICOLON)
            '!' -> if (match('=')) addToken(TokenKind.NEQ) else addToken(TokenKind.NOT)
            '=' -> if (match('=')) addToken(TokenKind.EQEQ) else addToken(TokenKind.ASSIGN)
            '<' -> if (match('=')) addToken(TokenKind.LE) else addToken(TokenKind.LT)
            '>' -> if (match('=')) addToken(TokenKind.GE) else addToken(TokenKind.GT)
            '&' -> if (match('&')) addToken(TokenKind.ANDAND) else throw LexError("Unexpected character '&'", line, col)
            '|' -> if (match('|')) addToken(TokenKind.OROR) else throw LexError("Unexpected character '|'", line, col)
            else -> throw LexError("Unexpected character '$c'", line, col)
        }
    }

    private fun addToken(kind: TokenKind, lexeme: String = source.substring(start, current)) {
        tokens.add(Token(kind, lexeme, line, col))
    }

    private fun match(expected: Char): Boolean {
        if (isAtEnd() || source[current] != expected) return false
        advance(); return true
    }

    private fun peek(): Char = if (isAtEnd()) '\u0000' else source[current]
    private fun peekNext(): Char = if (current + 1 >= source.length) '\u0000' else source[current + 1]
    private fun isAtEnd(): Boolean = current >= source.length
    private fun advance(): Char { current++; col++; return source[current - 1] }
}

