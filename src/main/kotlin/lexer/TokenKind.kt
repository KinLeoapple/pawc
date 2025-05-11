package org.pawscript.lexer

// --- Token Definitions ---
enum class TokenKind {
    // Single-character tokens
    PLUS, MINUS, STAR, SLASH, PERCENT, QUESTION,
    LPAREN, RPAREN, LBRACE, RBRACE, LBRACKET, RBRACKET,
    COMMA, DOT, COLON, SEMICOLON,
    // One or two character tokens
    EQ, EQEQ, NEQ, LT, LE, GT, GE,
    ANDAND, OROR, NOT, ASSIGN, ARROW, RANGE,
    // Literals
    IDENTIFIER, INT_LITERAL, FLOAT_LITERAL, STRING_LITERAL, CHAR_LITERAL,
    TEMPLATE_TEXT, TEMPLATE_EXPR_START, TEMPLATE_EXPR_END,
    NOPAW,
    // Keywords
    SAY, ASK,
    LET, FUN, IF, ELSE, LOOP, IN, RETURN, BARK, ASYNC, AWAIT, IMPORT, AS,
    RECORD, TAIL, SNIFF, SNATCH, LASTLY,
    BREAK, CONTINUE,
    EOF
}