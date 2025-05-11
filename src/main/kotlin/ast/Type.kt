package org.pawscript.ast

sealed class Type : Node {
    data class Custom(val name: kotlin.String) : Type()

    object Int : Type()
    object Long : Type()
    object Float : Type()
    object Double : Type()
    object Bool : Type()
    object Char : Type()
    object String : Type()
    object Void : Type()
    object Any : Type()
    data class Array(val elementType: Type) : Type()
    data class Optional(val baseType: Type) : Type()
    data class Module(val name: kotlin.String) : Type()
    data class Future(val baseType: Type) : Type()
    data class Generic(val baseName: kotlin.String, val typeArgs: List<Type>) : Type()
    data class TypeVar(val name: kotlin.String) : Type()
}