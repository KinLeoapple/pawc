package org.pawscript.lexer

data class Token(
    val kind: TokenKind,
    val lexeme: String,
    val line: Int,
    val col: Int
)