package org.pawscript.ast

sealed class Pattern : Node {
    /** 通配符 `_` */
    object Wildcard : Pattern()

    /** 绑定变量 `x` */
    data class Var(val name: String) : Pattern()

    /** 字面量匹配：Int/String/Bool/Char */
    data class Literal(val value: Any) : Pattern()

    /** 构造器模式：Record 或 Module 函数等 */
    data class Constructor(
        val name: String,
        val args: List<Pattern>
    ) : Pattern()
}