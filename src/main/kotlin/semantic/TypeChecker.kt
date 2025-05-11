package org.pawscript.semantic

import org.pawscript.ast.*
import org.pawscript.ast.BinaryOp.*
import org.pawscript.ast.UnaryOp.*
import org.pawscript.error.PawError.*

class TypeChecker {
    // Environments
    private val interfaces = mutableMapOf<String, Declaration.Tail>()
    private val records    = mutableMapOf<String, Declaration.Record>()
    private val functions  = mutableMapOf<String, Declaration.Fun>()
    private val imports = mutableMapOf<String, String>()
    private val varScopes  = mutableListOf<MutableMap<String, Type>>()
    private val inLoopScopes = mutableListOf<Boolean>()
    private var loopDepth: Int = 0
    private val typeParamStack = mutableListOf<List<String>>()
    private val currentTypeParams: List<String>
        get() = typeParamStack.lastOrNull() ?: emptyList()

    /** Entry: collect imports, declarations, then check all nodes */
    fun check(nodes: List<Node>) {
        // second: declarations
        for (n in nodes) {
            when (n) {
                is Declaration.Import -> registerImport(n)
                is Declaration.Tail   -> registerInterface(n)
                is Declaration.Record -> registerRecord(n)
                is Declaration.Fun    -> registerFunction(n)
                else -> {}
            }
        }
        // third: check declarations and statements
        for (n in nodes) {
            when (n) {
                is Declaration.Fun    -> checkFunction(n)
                is Declaration.Record -> validateRecordImplementation(n)
                is Statement -> checkStatement(n)
                else -> {}
            }
        }
    }

    // --- Import ---
    private fun registerImport(d: Declaration.Import) {
        val alias = d.alias ?: d.module
        if (imports[alias] != null) {
            throw DeclarationError("Module alias '$alias' already defined", 0, 0)
        }
        imports[alias] = d.module
    }

    // --- Declaration registration ---
    private fun registerInterface(d: Declaration.Tail) {
        if (interfaces.contains(d.name))
            throw DuplicateDeclarationError("Interface '${d.name}' already declared", 0, 0)
        interfaces[d.name] = d
    }

    private fun registerRecord(d: Declaration.Record) {
        if (records.contains(d.name))
            throw DuplicateDeclarationError("Record '${d.name}' already declared", 0, 0)
        records[d.name] = d
    }

    private fun registerFunction(d: Declaration.Fun) {
        // 如果有 receiverType，就把 key 设为 "Receiver.Name"，否则就是普通函数名
        val key = if (d.receiverType != null) {
            "${d.receiverType}.${d.name}"
        } else {
            d.name
        }
        if (functions.containsKey(key)) {
            throw DuplicateDeclarationError("Function '$key' already declared", 0, 0)
        }
        functions[key] = d
    }

    // --- Validate record implements interfaces if specified ---
    private fun validateRecordImplementation(d: Declaration.Record) {
        for (ifaceName in d.implements) {
            val iface = interfaces[ifaceName]
                ?: throw DeclarationError("Unknown interface '$ifaceName' in record '${d.name}'", 0, 0)
            for (sig in iface.methods) {
                val key = "${d.name}.${sig.name}"
                val impl = functions[key]
                    ?: throw DeclarationError(
                        "Record '${d.name}' must implement '${sig.name}' of interface '$ifaceName'",
                        0, 0
                    )
                // 检查参数列表长度
                if (impl.params.size != sig.params.size) {
                    throw DeclarationError(
                        "Signature mismatch for method '${sig.name}' in '${d.name}'",
                        0, 0
                    )
                }
                // 检查每个参数类型，跳过 'self'
                impl.params.zip(sig.params).forEach { (pImpl, pSig) ->
                    if (pSig.name == "self") return@forEach
                    if (pImpl.type != pSig.type) {
                        throw DeclarationError(
                            "In implementation of '${sig.name}', parameter '${pImpl.name}' should be ${pSig.type}",
                            0, 0
                        )
                    }
                }
                // 检查返回类型
                if (impl.returnType != sig.returnType) {
                    throw DeclarationError(
                        "In implementation of '${sig.name}', return type should be ${sig.returnType}",
                        0, 0
                    )
                }
            }
        }
    }

    // --- Check a function ---
    private fun checkFunction(f: Declaration.Fun) {
        enterScope()
        f.params.forEach { declareVar(it.name, it.type) }
        f.body.forEach { checkStatement(it) }
        exitScope()
    }

    // --- Statement checking ---
    private fun checkStatement(stmt: Statement) {
        when (stmt) {
            is Statement.Let -> {
                val tExpr = checkExpr(stmt.expr)
                val tVar = stmt.type ?: tExpr
                if (stmt.type != null && tExpr != tVar)
                    throw TypeError("Let '${stmt.name}' expects $tVar but got $tExpr", 0, 0)
                declareVar(stmt.name, tVar)
            }
            is Statement.Assign -> {
                val tVar = resolveVar(stmt.name)
                val tExpr = checkExpr(stmt.expr)
                if (tVar != tExpr)
                    throw TypeError("Assign to '${stmt.name}' expects $tVar but got $tExpr", 0, 0)
            }
            is Statement.If -> {
                val cond = checkExpr(stmt.condition)
                if (cond != Type.Bool)
                    throw TypeError("If condition must be Bool, got $cond", 0, 0)
                stmt.thenBranch.forEach { checkStatement(it) }
                stmt.elseBranch?.forEach { checkStatement(it) }
            }
            // 无限循环
            is Statement.LoopInfinite -> {
                loopDepth++
                enterScope()
                stmt.body.forEach { checkStatement(it) }
                exitScope()
                loopDepth--
            }

            // while 循环（无新变量）
            is Statement.LoopWhile -> {
                if (checkExpr(stmt.condition) != Type.Bool) {
                    throw TypeError("Loop condition must be Bool", 0, 0)
                }
                loopDepth++
                enterScope()
                stmt.body.forEach { checkStatement(it) }
                exitScope()
                loopDepth--
            }

            // 简单数组循环： loop x in expr
            is Statement.LoopIn -> {
                // 检查集合类型略…
                loopDepth++
                enterScope()
                // 声明循环变量 x
                declareVar(stmt.itemName, /* 可根据 expr 推断元素类型，暂用 Any */ Type.Any)
                stmt.body.forEach { checkStatement(it) }
                exitScope()
                loopDepth--
            }

            // 索引数组循环： loop idx, x in expr
            is Statement.LoopIndexed -> {
                loopDepth++
                enterScope()
                declareVar(stmt.indexName, Type.Int)  // 索引总是 Int
                declareVar(stmt.itemName,  Type.Any)  // 元素类型可做推断，这里先用 Any
                stmt.body.forEach { checkStatement(it) }
                exitScope()
                loopDepth--
            }

            // 范围循环： loop x in start..end
            is Statement.LoopRange -> {
                if (checkExpr(stmt.start) !in listOf(Type.Int, Type.Float)) {
                    throw TypeError("Range bounds must be numeric", 0, 0)
                }
                if (checkExpr(stmt.end) !in listOf(Type.Int, Type.Float)) {
                    throw TypeError("Range bounds must be numeric", 0, 0)
                }
                loopDepth++
                enterScope()
                declareVar(stmt.variable, Type.Int)  // 迭代变量通常是 Int
                stmt.body.forEach { checkStatement(it) }
                exitScope()
                loopDepth--
            }

            // break / continue 校验…
            is Statement.Break, is Statement.Continue -> {
                if (loopDepth == 0) {
                    throw ControlFlowError(
                        "${if (stmt is Statement.Break) "break" else "continue"} outside of loop",
                        0, 0
                    )
                }
            }
            is Statement.Return -> stmt.expr?.let { checkExpr(it) }
            is Statement.Bark -> checkExpr(stmt.expr)
            is Statement.Say -> {
                val t = checkExpr(stmt.expr)
                if (t != Type.String)
                    throw TypeError("say requires a String expression, got $t", 0, 0)
            }
            is Statement.Ask -> {
                // ask returns a String
            }
            is Statement.ExprStmt -> checkExpr(stmt.expr)
            is Statement.Sniff -> {
                stmt.tryBlock.forEach { checkStatement(it) }
                stmt.catchBlock?.forEach { checkStatement(it) }
                stmt.finallyBlock?.forEach { checkStatement(it) }
            }
        }
    }

    // --- Expression checking ---
    private fun checkExpr(expr: Expr): Type {
        return when (expr) {
            is Expr.LiteralInt    -> Type.Int
            is Expr.LiteralFloat  -> Type.Float
            is Expr.LiteralBool   -> Type.Bool
            is Expr.LiteralChar   -> Type.Char
            is Expr.LiteralString -> Type.String

            is Expr.AskExpr -> Type.String

            Expr.Nopaw            -> Type.Optional(Type.Void)

            is Expr.Variable      -> resolveVar(expr.name)

            is Expr.Binary        -> checkBinary(expr)
            is Expr.Unary         -> checkUnary(expr)
            is Expr.StringTemplate-> Type.String

            // 函数调用
            is Expr.Call -> {
                // 1) 找到函数声明
                val decl = functions[expr.callee]
                    ?: throw NameError("Undefined function '${expr.callee}'", 0, 0)

                // 2) 检查泛型实参数量
                if (decl.typeParams.size != expr.typeArgs.size) {
                    throw FunctionCallError(
                        "Function '${decl.name}' expects ${decl.typeParams.size} type arguments, got ${expr.typeArgs.size}",
                        0, 0
                    )
                }

                // 3) 对每个 typeArg 做类型检查（注：TypeChecker.checkExpr 不适合直接接受 Type AST，使用这个辅助）
                val checkedTypeArgs = expr.typeArgs.map { t ->
                    // 允许泛型实参里也出现泛型嵌套等结构
                    checkTypeAnnotation(t)
                }

                // 4) 建立类型变量 -> 实参 类型环境
                val typeEnv: Map<String,Type> = decl.typeParams
                    .zip(checkedTypeArgs)
                    .toMap()

                // 5) 实例化参数类型和返回类型
                val instantiatedParamTypes: List<Type> = decl.params.map { p ->
                    substitute(p.type, typeEnv)
                }
                val instantiatedReturnType: Type? = decl.returnType?.let { substitute(it, typeEnv) }

                // 6) 参数数量检查
                val totalArgs = expr.positional.size + expr.named.size
                if (totalArgs != instantiatedParamTypes.size) {
                    throw FunctionCallError(
                        "Function '${decl.name}' expects ${instantiatedParamTypes.size} args, got $totalArgs",
                        0, 0
                    )
                }

                // 7) 逐一类型检查实参
                for ((i, expectedType) in instantiatedParamTypes.withIndex()) {
                    val argExpr = if (i < expr.positional.size) {
                        expr.positional[i]
                    } else {
                        val name = decl.params[i].name
                        expr.named[name]
                            ?: throw FunctionCallError("Missing argument for parameter '$name'", 0, 0)
                    }
                    val actualType = checkExpr(argExpr)
                    if (actualType != expectedType) {
                        throw FunctionCallError(
                            "In call to '${decl.name}', parameter '${decl.params[i].name}' expects $expectedType but got $actualType",
                            0, 0
                        )
                    }
                }

                // 8) 返回类型
                return if (decl.isAsync) {
                    Type.Future(instantiatedReturnType
                        ?: throw FunctionCallError("Async function must declare return type", 0, 0)
                    )
                } else {
                    instantiatedReturnType
                        ?: throw FunctionCallError("Function '${decl.name}' must declare return type", 0, 0)
                }
            }
            is Expr.MethodCall -> {
                // 1) 检查目标对象类型
                val tTarget = checkExpr(expr.target)
                val key = when (tTarget) {
                    is Type.Custom -> "${tTarget.name}.${expr.method}"
                    else -> throw TypeError("Cannot call method '${expr.method}' on type $tTarget", 0, 0)
                }
                val decl = functions[key]
                    ?: throw NameError("Undefined method '${expr.method}' on type $tTarget", 0, 0)

                // 2) 弹掉 decl.params 中第一个 self
                val expectedParams = decl.params.drop(1)

                // 3) 检查调用时传入的参数数目
                val totalArgs = expr.positional.size + expr.named.size
                if (totalArgs != expectedParams.size) {
                    throw FunctionCallError(
                        "Method '$key' expects ${expectedParams.size} args, got $totalArgs", 0, 0
                    )
                }

                // 4) 按顺序或按名匹配参数类型
                for ((i, param) in expectedParams.withIndex()) {
                    val argExpr = if (i < expr.positional.size) {
                        expr.positional[i]
                    } else {
                        expr.named[param.name]
                            ?: throw FunctionCallError("Missing argument for parameter '${param.name}'", 0, 0)
                    }
                    val tArg = checkExpr(argExpr)
                    if (tArg != param.type) {
                        throw FunctionCallError(
                            "In method '$key', parameter '${param.name}' expects ${param.type} but got $tArg",
                            0, 0
                        )
                    }
                }

                // 5) 返回类型：async 方法封装成 Future
                if (decl.isAsync) {
                    Type.Future(decl.returnType
                        ?: throw FunctionCallError("Async method '$key' must declare return type", 0, 0)
                    )
                } else {
                    decl.returnType
                        ?: throw FunctionCallError("Method '$key' must declare return type", 0, 0)
                }
            }

            // 对象字段访问
            is Expr.FieldAccess -> {
                val targetType = checkExpr(expr.target)
                val recordDecl = when (targetType) {
                    is Type.Custom -> records[targetType.name]
                    is Type.Module -> null
                    else           -> null
                } ?: throw FieldAccessError(
                    "Type '$targetType' has no fields", 0, 0
                )
                // 找到字段声明
                val fieldParam = recordDecl.fields.find { it.name == expr.field }
                    ?: throw FieldAccessError(
                        "Field '${expr.field}' not found on record '${recordDecl.name}'", 0, 0
                    )
                fieldParam.type
            }

            // 数组字面量
            is Expr.ArrayLiteral -> {
                if (expr.elements.isEmpty()) {
                    throw TypeError("Cannot infer element type of empty array literal", 0, 0)
                }
                // 强制所有元素同类型
                val firstType = checkExpr(expr.elements[0])
                for (e in expr.elements.drop(1)) {
                    val t = checkExpr(e)
                    if (t != firstType) {
                        throw TypeError("Array elements must all be same type, got $firstType and $t", 0, 0)
                    }
                }
                Type.Array(firstType)
            }

            // 类型转换 expr as Type
            is Expr.As -> {
                val source = checkExpr(expr.expr)
                val target = expr.targetType
                if (!canCast(source, target)) {
                    throw TypeConversionError("Cannot cast from $source to $target", 0, 0)
                }
                target
            }

            // await expr
            is Expr.Await -> {
                val awaitedType = checkExpr(expr.expr)
                if (awaitedType is Type.Future) {
                    // await Future<T> yields T
                    awaitedType.baseType
                } else {
                    throw AsyncError(
                        "await can only be used on Future<T> results, got $awaitedType",
                        0, 0
                    )
                }
            }

            else -> throw TypeError("Unsupported expression type: ${expr::class.simpleName}", 0, 0)
        }
    }

    /** 简单的类型转换规则示例 */
    private fun canCast(src: Type, dest: Type): Boolean {
        // 同类型直接通
        if (src == dest) return true
        // Int -> Float
        if (src == Type.Int && dest == Type.Float) return true
        // 基本可空转换： T -> Optional<T>
        if (dest is Type.Optional && src == dest.baseType) return true
        return false
    }

    private fun checkTypeAnnotation(t: Type): Type {
        // 对 Generic 里的每个子类型递归检查
        when (t) {
            is Type.Generic -> t.typeArgs.forEach { checkTypeAnnotation(it) }
            is Type.Array   -> checkTypeAnnotation(t.elementType)
            is Type.Optional-> checkTypeAnnotation(t.baseType)
            is Type.Future  -> checkTypeAnnotation(t.baseType)
            is Type.TypeVar -> {
                if (t.name !in currentTypeParams) {
                    throw TypeError("Unknown type variable '${t.name}'", 0, 0)
                }
            }
            is Type.Custom  -> {
                // 验证这个自定义类型名要么是接口名，要么是记录名
                if (!interfaces.containsKey(t.name) && !records.containsKey(t.name)) {
                    throw NameError("Unknown type '${t.name}'", 0, 0)
                }
            }
            else -> {}
        }
        return t
    }

    private fun checkBinary(e: Expr.Binary): Type {
        val lt = checkExpr(e.left)
        val rt = checkExpr(e.right)
        return when (e.op) {
            ADD, SUB, MUL, DIV, MOD -> {
                if (lt == rt && (lt == Type.Int || lt == Type.Float)) lt
                else throw TypeError("${e.op} requires Int/Float, got $lt and $rt", 0, 0)
            }
            EQEQ, NEQ, LT, LE, GT, GE -> Type.Bool
            AND, OR -> {
                if (lt == Type.Bool && rt == Type.Bool) Type.Bool
                else throw TypeError("${e.op} requires Bool, got $lt and $rt", 0, 0)
            }
        }
    }

    private fun checkUnary(e: Expr.Unary): Type {
        val t = checkExpr(e.expr)
        return when (e.op) {
            NOT -> if (t == Type.Bool) Type.Bool else throw TypeError("! requires Bool, got $t",0,0)
            NEG -> if (t == Type.Int) Type.Int else throw TypeError("- requires Int, got $t",0,0)
        }
    }

    // --- Helpers ---
    private fun enterScope() { varScopes.add(mutableMapOf()) }
    private fun exitScope()  { varScopes.removeAt(varScopes.lastIndex) }

    private fun enterLoop()   { inLoopScopes.add(true) }
    private fun exitLoop()    { inLoopScopes.removeAt(inLoopScopes.lastIndex) }
    private fun inLoop(): Boolean = inLoopScopes.isNotEmpty()

    private fun declareVar(name: String, type: Type) {
        if (varScopes.isEmpty()) varScopes.add(mutableMapOf())
        val scope = varScopes.last()
        if (scope.containsKey(name)) {
            throw DuplicateDeclarationError("Variable '$name' already declared", 0, 0)
        }
        scope[name] = type
    }

    private fun resolveVar(name: String): Type {
        for (i in varScopes.size - 1 downTo 0) {
            varScopes[i][name]?.let { return it }
        }
        imports[name]?.let { moduleName ->
            return Type.Module(moduleName)
        }
        throw NameError("Undefined variable '$name'", 0, 0)
    }

    private fun substitute(type: Type, env: Map<String,Type>): Type = when(type) {
        is Type.TypeVar -> env[type.name] ?: type
        is Type.Generic -> {
            val newArgs = type.typeArgs.map { substitute(it, env) }
            Type.Generic(type.baseName, newArgs)
        }
        is Type.Array    -> Type.Array(substitute(type.elementType, env))
        is Type.Optional -> Type.Optional(substitute(type.baseType, env))
        is Type.Future   -> Type.Future(substitute(type.baseType, env))
        else             -> type  // Custom, Int, String, etc.
    }
}
