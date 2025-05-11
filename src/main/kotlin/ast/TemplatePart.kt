package org.pawscript.ast

sealed class TemplatePart : Node {
    data class Text(val text: String) : TemplatePart()
    data class ExprPart(val expr: Expr) : TemplatePart()
}