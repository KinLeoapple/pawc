package org.pawscript.ast

sealed class Statement: Node {
    data class Let(val name: String, val type: Type?, val expr: Expr) : Statement()
    data class Assign(val name: String, val expr: Expr) : Statement()
    data class Say(val expr: Expr) : Statement()
    data class Ask(val prompt: String) : Statement()
    data class Return(val expr: Expr?) : Statement()
    data class Bark(val expr: Expr) : Statement()
    data class ExprStmt(val expr: Expr) : Statement()
    data class If(
        val condition: Expr,
        val thenBranch: List<Statement>,
        val elseBranch: List<Statement>?
    ) : Statement()
    data class LoopInfinite(val body: List<Statement>) : Statement()
    data class LoopWhile(val condition: Expr, val body: List<Statement>) : Statement()
    data class LoopRange(
        val variable: String,
        val start: Expr,
        val end: Expr,
        val body: List<Statement>
    ) : Statement()
    data class LoopIn(
        val itemName: String,
        val arrayExpr: Expr,
        val body: List<Statement>
    ) : Statement()
    data class LoopIndexed(
        val indexName: String,
        val itemName: String,
        val arrayExpr: Expr,
        val body: List<Statement>
    ) : Statement()
    data class Sniff(
        val tryBlock: List<Statement>,
        val catchVar: String?,             // snatch 后的异常变量名
        val catchBlock: List<Statement>?,  // snatch 块
        val finallyBlock: List<Statement>? // lastly 块
    ): Statement()
    object Break : Statement()
    object Continue : Statement()
}
