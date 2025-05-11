package org.pawscript.ast

data class MethodSig(
    val name: String,
    val params: List<Param>,
    val returnType: Type
)
