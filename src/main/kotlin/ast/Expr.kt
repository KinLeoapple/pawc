package org.pawscript.ast

sealed class Expr : Node {
    data class LiteralInt(val value: Long) : Expr()
    data class LiteralFloat(val value: Double) : Expr()
    data class LiteralBool(val value: Boolean) : Expr()
    data class LiteralChar(val value: Char) : Expr()
    data class LiteralString(val value: String) : Expr()
    data class ArrayLiteral(val elements: List<Expr>) : Expr()
    object Nopaw : Expr()
    data class Variable(val name: String) : Expr()
    data class Binary(val left: Expr, val op: BinaryOp, val right: Expr) : Expr()
    data class Unary(val op: UnaryOp, val expr: Expr) : Expr()
    data class StringTemplate(val parts: List<TemplatePart>) : Expr()
    data class Call(
        val callee: String,
        val typeArgs: List<Type>,
        val positional: List<Expr>,
        val named: Map<String, Expr>
    ) : Expr()

    data class MethodCall(
        val target: Expr,
        val method: String,
        val positional: List<Expr>,
        val named: Map<String, Expr>
    ) : Expr()
    data class FieldAccess(val target: Expr, val field: String) : Expr()
    data class ModuleAccess(val moduleAlias: String, val member: String) : Expr()
    data class As(val expr: Expr, val targetType: Type) : Expr()
    data class Await(val expr: Expr) : Expr()
    data class AskExpr(val prompt: String) : Expr()
    data class Match(
        val scrutinee: Expr,        // 要匹配的目标
        val cases: List<Case>        // 一组 case 分支
    ) : Expr()
    data class Case(
        val pattern: Pattern,
        val result: Expr
    ) : Node
    data class BlockExpr(val statements: List<Statement>) : Expr()
}