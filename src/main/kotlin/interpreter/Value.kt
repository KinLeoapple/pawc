package org.pawscript.interpreter

import kotlinx.coroutines.Deferred

sealed class Value {
    data class IntVal(   val v: Int)                   : Value()
    data class FloatVal( val v: Double)                : Value()
    data class BoolVal(  val v: Boolean)               : Value()
    data class StrVal(   val v: String)                : Value()
    object    NullVal                                   : Value()
    data class ArrayVal(val elems: List<Value>)        : Value()
    data class RecordVal(
        val name: String,
        val fields: Map<String, Value>
    ) : Value()
    data class FutureVal(val deferred: Deferred<Value>) : Value()
    data class FunctionVal(
        val fn: suspend (args: List<Value>) -> Value
    ) : Value()
    data class ModuleVal(
        val functions: Map<String, FunctionVal>,
        val constants: Map<String, Value>
    ) : Value()

    override fun toString(): String = when (this) {
        is IntVal    -> v.toString()
        is FloatVal  -> v.toString()
        is BoolVal   -> v.toString()
        is StrVal    -> v
        is NullVal   -> "null"
        is ArrayVal  -> elems.joinToString(prefix = "[", postfix = "]")
        is RecordVal -> "<${name} ${fields}>"
        is FutureVal -> "<Future>"
        is FunctionVal -> "<Function>"
        is ModuleVal   -> "<Module>"
    }
}
