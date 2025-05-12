package org.pawscript.interpreter

class Env(
    private val parent: Env? = null
) {
    private val values: MutableMap<String, Value> = mutableMapOf()

    /** 创建当前环境的子环境 */
    fun child(): Env = Env(this)

    /**
     * 定义新标识符（变量、函数或模块），仅在当前作用域。
     * 如果已存在则抛出异常。
     */
    fun define(name: String, value: Value) {
        if (values.containsKey(name)) {
            error("Identifier '$name' already defined in current scope")
        }
        values[name] = value
    }

    /**
     * 更新已定义标识符的值。如果当前作用域没有，则向上查找；
     * 找不到则抛异常。
     */
    fun assign(name: String, value: Value) {
        when {
            values.containsKey(name) -> values[name] = value
            parent != null           -> parent.assign(name, value)
            else                     -> error("Undefined identifier '$name'")
        }
    }

    /**
     * 查找标识符的当前值，从当前作用域向上查找。
     * 找不到则抛异常。
     */
    fun lookup(name: String): Value {
        return values[name]
            ?: parent?.lookup(name)
            ?: error("Undefined identifier '$name'")
    }

    /**
     * 判断标识符是否在当前或父环境中定义。
     */
    fun contains(name: String): Boolean {
        return values.containsKey(name) || parent?.contains(name) == true
    }

    /**
     * 列出当前作用域所有标识符（不包括父作用域）。
     */
    fun keys(): Set<String> = values.keys
}
