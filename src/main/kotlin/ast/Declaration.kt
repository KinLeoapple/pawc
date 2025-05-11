package org.pawscript.ast

sealed class Declaration : Node {
    data class Tail(
        val name: String,
        val methods: List<MethodSig>
    ) : Declaration()
    data class Fun(
        val typeParams: List<String>,
        val receiverType: String? = null,
        val name: String,
        val params: List<Param>,
        val returnType: Type?,
        val body: List<Statement>,
        val isAsync: Boolean = false
    ) : Declaration()
    data class Record(
        val name: String,
        val implements: List<String>,
        val fields: List<Param>
    ) : Declaration()
    data class Import(
        val module: String,
        val alias: String?
    ) : Declaration()
}