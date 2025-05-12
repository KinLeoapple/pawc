package org.pawscript.semantic

import org.pawscript.ast.*
import org.pawscript.ast.BinaryOp.*
import org.pawscript.ast.UnaryOp.*
import org.pawscript.error.PawError.*

class TypeChecker(
    private val externalModules: Map<String, ModuleContents>
) {
    // --- 符号表 ---
    private val imports = mutableMapOf<String, String>()
    private val functions = mutableMapOf<String, Declaration.Fun>()
    private val constants = mutableMapOf<String, Type>()
    private val interfaces = mutableMapOf<String, Declaration.Tail>()
    private val records = mutableMapOf<String, Declaration.Record>()
    private val varScopes = mutableListOf<MutableMap<String, Type>>()
    private val typeParamStack = mutableListOf<List<String>>()
    private val currentTypeParams: List<String>
        get() = typeParamStack.lastOrNull() ?: emptyList()
    private var loopDepth = 0

    /** 入口：先导入模块，其次注册声明，最后做类型检查 */
    fun check(nodes: List<Node>) {
        // 1. 处理 import，注入外部模块符号
        for (n in nodes) {
            if (n is Declaration.Import) registerImport(n)
        }
        // 2. 注册本地接口、记录、函数
        for (n in nodes) {
            when (n) {
                is Declaration.Tail -> registerInterface(n)
                is Declaration.Record -> registerRecord(n)
                is Declaration.Fun -> registerLocalFunction(n)
                else -> {}
            }
        }
        // 3. 验证记录实现接口
        for (r in records.values) {
            validateRecordImplementation(r)
        }
        // 4. 对所有函数和语句执行类型检查
        for (n in nodes) {
            when (n) {
                is Declaration.Fun -> checkFunction(n)
                is Statement -> checkStatement(n)
                else -> {}
            }
        }
    }

    // --- Import 注册 ---
    private fun registerImport(d: Declaration.Import) {
        val alias = d.alias ?: d.module
        if (imports.containsKey(alias)) {
            throw DeclarationError("Module alias '$alias' already defined", 0, 0)
        }
        imports[alias] = d.module

        val mod = externalModules[d.module]
            ?: throw DeclarationError("Unknown module '${d.module}'", 0, 0)

        // 注入函数
        for (fn in mod.functions) {
            if (!functions.containsKey(fn.name)) {
                functions[fn.name] = fn
            }
            val qfn = "$alias.${fn.name}"
            if (functions.containsKey(qfn)) {
                throw DeclarationError("Function '$qfn' already declared", 0, 0)
            }
            functions[qfn] = fn
        }
        // 注入常量
        for ((cn, ct) in mod.constants) {
            if (!constants.containsKey(cn)) {
                constants[cn] = ct
            }
            val qcn = "$alias.$cn"
            if (constants.containsKey(qcn)) {
                throw DeclarationError("Constant '$qcn' already declared", 0, 0)
            }
            constants[qcn] = ct
        }
    }

    // --- 本地声明注册 ---
    private fun registerInterface(d: Declaration.Tail) {
        if (interfaces.containsKey(d.name)) {
            throw DuplicateDeclarationError("Interface '${d.name}' already declared", 0, 0)
        }
        interfaces[d.name] = d
    }

    private fun registerRecord(d: Declaration.Record) {
        if (records.containsKey(d.name)) {
            throw DuplicateDeclarationError("Record '${d.name}' already declared", 0, 0)
        }
        records[d.name] = d

        val ctor = Declaration.Fun(
            typeParams = emptyList(),
            receiverType = null,
            name = d.name,
            params = d.fields,
            returnType = Type.Custom(d.name),
            body = emptyList(),
            isAsync = false
        )

        if (functions.containsKey(d.name)) {
            throw DuplicateDeclarationError("Function '${d.name}' already declared", 0, 0)
        }
        functions[d.name] = ctor

        for ((alias, _) in imports) {
            val qname = "$alias.${d.name}"
            if (functions.containsKey(qname)) {
                throw DuplicateDeclarationError("Function '$qname' already declared", 0, 0)
            }
            functions[qname] = ctor
        }
    }

    private fun registerLocalFunction(d: Declaration.Fun) {
        if (functions.containsKey(d.name)) {
            throw DuplicateDeclarationError("Function '${d.name}' already declared", 0, 0)
        }
        functions[d.name] = d

        d.receiverType?.let { recv ->
            val key = "$recv.${d.name}"
            if (functions.containsKey(key)) {
                throw DuplicateDeclarationError("Method '$key' already declared", 0, 0)
            }
            functions[key] = d
        }
    }

    // --- 验证记录实现接口 ---
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
                if (impl.params.size != sig.params.size) {
                    throw DeclarationError(
                        "Signature mismatch for method '${sig.name}' in '${d.name}'",
                        0, 0
                    )
                }
                impl.params.zip(sig.params).forEach { (pImpl, pSig) ->
                    if (pSig.name == "self") return@forEach
                    if (pImpl.type != pSig.type) {
                        throw DeclarationError(
                            "In implementation of '${sig.name}', parameter '${pImpl.name}' should be ${pSig.type}",
                            0, 0
                        )
                    }
                }
                if (impl.returnType != sig.returnType) {
                    throw DeclarationError(
                        "In implementation of '${sig.name}', return type should be ${sig.returnType}",
                        0, 0
                    )
                }
            }
        }
    }

    // --- 函数检查 ---
    private fun checkFunction(f: Declaration.Fun) {
        // 处理泛型类型参数范围
        typeParamStack.add(f.typeParams)
        enterScope()
        f.params.forEach { declareVar(it.name, it.type) }
        f.body.forEach { checkStatement(it) }
        exitScope()
        typeParamStack.removeAt(typeParamStack.lastIndex)
    }

    // --- 语句检查 ---
    private fun checkStatement(stmt: Statement) {
        when (stmt) {
            is Statement.Let -> {
                val tExpr = checkExpr(stmt.expr)
                val tVar = stmt.type ?: tExpr
                if (stmt.type != null && tExpr != tVar) {
                    throw TypeError("Let '${stmt.name}' expects $tVar but got $tExpr", 0, 0)
                }
                declareVar(stmt.name, tVar)
            }

            is Statement.Assign -> {
                val tVar = resolveVar(stmt.name)
                val tExpr = checkExpr(stmt.expr)
                if (tVar != tExpr) {
                    throw TypeError("Assign to '${stmt.name}' expects $tVar but got $tExpr", 0, 0)
                }
            }

            is Statement.If -> {
                val cond = checkExpr(stmt.condition)
                if (cond != Type.Bool) {
                    throw TypeError("If condition must be Bool, got $cond", 0, 0)
                }
                stmt.thenBranch.forEach { checkStatement(it) }
                stmt.elseBranch?.forEach { checkStatement(it) }
            }

            is Statement.LoopInfinite -> {
                loopDepth++; enterScope()
                stmt.body.forEach { checkStatement(it) }
                exitScope(); loopDepth--
            }

            is Statement.LoopWhile -> {
                if (checkExpr(stmt.condition) != Type.Bool) {
                    throw TypeError("Loop condition must be Bool", 0, 0)
                }
                loopDepth++; enterScope()
                stmt.body.forEach { checkStatement(it) }
                exitScope(); loopDepth--
            }

            is Statement.LoopIn -> {
                loopDepth++; enterScope()
                declareVar(stmt.itemName, Type.Any)
                stmt.body.forEach { checkStatement(it) }
                exitScope(); loopDepth--
            }

            is Statement.LoopIndexed -> {
                // 先判断被遍历的表达式类型
                val tIter = checkExpr(stmt.arrayExpr)
                val elemType = when (tIter) {
                    is Type.Array -> tIter.elementType
                    else -> throw TypeError(
                        "Cannot loop over non-array type $tIter",
                        0, 0
                    )
                }

                loopDepth++
                enterScope()
                // 索引变量是 Int
                declareVar(stmt.indexName, Type.Int)
                // 元素变量用推断出来的 elemType
                declareVar(stmt.itemName, elemType)
                stmt.body.forEach { checkStatement(it) }
                exitScope()
                loopDepth--
            }

            is Statement.LoopRange -> {
                if (checkExpr(stmt.start) !in listOf(Type.Int, Type.Float) ||
                    checkExpr(stmt.end) !in listOf(Type.Int, Type.Float)
                ) {
                    throw TypeError("Range bounds must be numeric", 0, 0)
                }
                loopDepth++; enterScope()
                declareVar(stmt.variable, Type.Int)
                stmt.body.forEach { checkStatement(it) }
                exitScope(); loopDepth--
            }

            is Statement.Break, is Statement.Continue -> {
                if (loopDepth == 0) {
                    throw ControlFlowError(
                        if (stmt is Statement.Break) "break" else "continue" + " outside of loop",
                        0, 0
                    )
                }
            }

            is Statement.Return -> stmt.expr?.let { checkExpr(it) }
            is Statement.Bark -> checkExpr(stmt.expr)
            is Statement.Say -> {
                val t = checkExpr(stmt.expr)
                if (t != Type.String) {
                    throw TypeError("say requires a String expression, got $t", 0, 0)
                }
            }

            is Statement.Ask -> { /* ask returns String */
            }

            is Statement.ExprStmt -> checkExpr(stmt.expr)
            is Statement.Sniff -> {
                stmt.tryBlock.forEach { checkStatement(it) }
                stmt.catchBlock?.forEach { checkStatement(it) }
                stmt.finallyBlock?.forEach { checkStatement(it) }
            }
        }
    }

    // --- 表达式检查 ---
    private fun checkExpr(expr: Expr): Type {
        return when (expr) {
            is Expr.LiteralInt -> Type.Int
            is Expr.LiteralFloat -> Type.Float
            is Expr.LiteralBool -> Type.Bool
            is Expr.LiteralChar -> Type.Char
            is Expr.LiteralString -> Type.String
            is Expr.AskExpr -> Type.String
            Expr.Nopaw -> Type.Optional(Type.Void)
            is Expr.Variable -> resolveVar(expr.name)
            is Expr.StringTemplate -> Type.String

            is Expr.ModuleAccess -> {
                val alias = expr.moduleAlias
                val member = expr.member

                val modName = imports[alias]
                    ?: throw NameError("Unknown module alias '$alias'", 0, 0)

                constants["$alias.$member"]?.let { return it }

                val fn = functions["$alias.$member"]
                    ?: throw NameError("Module '$alias' has no constant or function '$member'", 0, 0)
                val paramTs = fn.params.map { it.type }
                val retT = fn.returnType
                    ?: throw FunctionCallError("Function '${fn.name}' must declare return type", 0, 0)
                return Type.Function(
                    typeParams = fn.typeParams,
                    paramTypes = paramTs,
                    returnType = retT,
                    isAsync = fn.isAsync
                )
            }

            is Expr.Call -> {
                val decl = functions[expr.callee]
                    ?: throw NameError("Undefined function '${expr.callee}'", 0, 0)
                if (decl.typeParams.size != expr.typeArgs.size) {
                    throw FunctionCallError(
                        "Function '${decl.name}' expects ${decl.typeParams.size} type arguments, got ${expr.typeArgs.size}",
                        0, 0
                    )
                }
                val checkedTypeArgs = expr.typeArgs.map { t -> checkTypeAnnotation(t) }
                val typeEnv = decl.typeParams.zip(checkedTypeArgs).toMap()
                val instantiatedParamTypes = decl.params.map { p -> substitute(p.type, typeEnv) }
                val instantiatedReturnType = decl.returnType?.let { substitute(it, typeEnv) }
                val totalArgs = expr.positional.size + expr.named.size
                if (totalArgs != instantiatedParamTypes.size) {
                    throw FunctionCallError(
                        "Function '${decl.name}' expects ${instantiatedParamTypes.size} args, got $totalArgs",
                        0, 0
                    )
                }
                for ((i, expected) in instantiatedParamTypes.withIndex()) {
                    val argExpr = if (i < expr.positional.size) expr.positional[i]
                    else expr.named[decl.params[i].name]
                        ?: throw FunctionCallError("Missing argument for parameter '${decl.params[i].name}'", 0, 0)
                    val actual = checkExpr(argExpr)
                    val ok = if (actual == expected) {
                        true
                    } else if (expected is Type.Optional && actual is Type.Optional) {
                        true
                    } else {
                        false
                    }

                    if (!ok) {
                        throw FunctionCallError(
                            "In call to '${decl.name}', parameter '${decl.params[i].name}' expects $expected but got $actual",
                            0, 0
                        )
                    }
                }
                if (decl.isAsync) {
                    Type.Future(
                        instantiatedReturnType
                            ?: throw FunctionCallError("Async function must declare return type", 0, 0)
                    )
                } else {
                    instantiatedReturnType
                        ?: throw FunctionCallError("Function '${decl.name}' must declare return type", 0, 0)
                }
            }

            is Expr.MethodCall -> {
                val tTarget = checkExpr(expr.target)
                val key = when (tTarget) {
                    is Type.Custom -> "${tTarget.name}.${expr.method}"
                    else -> throw TypeError("Cannot call method '${expr.method}' on type $tTarget", 0, 0)
                }
                val decl = functions[key]
                    ?: throw NameError("Undefined method '${expr.method}' on type $tTarget", 0, 0)
                val expectedParams = decl.params.drop(1)
                val totalArgs = expr.positional.size + expr.named.size
                if (totalArgs != expectedParams.size) {
                    throw FunctionCallError("Method '$key' expects ${expectedParams.size} args, got $totalArgs", 0, 0)
                }
                for ((i, param) in expectedParams.withIndex()) {
                    val argExpr = if (i < expr.positional.size) expr.positional[i]
                    else expr.named[param.name]
                        ?: throw FunctionCallError("Missing argument for parameter '${param.name}'", 0, 0)
                    val tArg = checkExpr(argExpr)
                    if (tArg != param.type) {
                        throw FunctionCallError(
                            "In method '$key', parameter '${param.name}' expects ${param.type} but got $tArg",
                            0, 0
                        )
                    }
                }
                if (decl.isAsync) {
                    Type.Future(
                        decl.returnType
                            ?: throw FunctionCallError("Async method '$key' must declare return type", 0, 0)
                    )
                } else {
                    decl.returnType
                        ?: throw FunctionCallError("Method '$key' must declare return type", 0, 0)
                }
            }

            is Expr.FieldAccess -> {
                val targetType = checkExpr(expr.target)
                val recordDecl = (targetType as? Type.Custom)?.let { records[it.name] }
                    ?: throw FieldAccessError("Type '$targetType' has no fields", 0, 0)
                val fieldParam = recordDecl.fields.find { it.name == expr.field }
                    ?: throw FieldAccessError("Field '${expr.field}' not found on record '${recordDecl.name}'", 0, 0)
                fieldParam.type
            }

            is Expr.ArrayLiteral -> {
                if (expr.elements.isEmpty()) {
                    throw TypeError("Cannot infer element type of empty array literal", 0, 0)
                }
                val ft = checkExpr(expr.elements[0])
                expr.elements.drop(1).forEach { e ->
                    val t = checkExpr(e)
                    if (t != ft) {
                        throw TypeError("Array elements must all be same type, got $ft and $t", 0, 0)
                    }
                }
                Type.Array(ft)
            }

            is Expr.As -> {
                val src = checkExpr(expr.expr)
                val tgt = expr.targetType
                if (!canCast(src, tgt)) {
                    throw TypeConversionError("Cannot cast from $src to $tgt", 0, 0)
                }
                tgt
            }

            is Expr.Await -> {
                val at = checkExpr(expr.expr)
                if (at is Type.Future) at.baseType
                else throw AsyncError("await can only be used on Future<T>, got $at", 0, 0)
            }

            is Expr.Binary -> checkBinary(expr)
            is Expr.Unary -> checkUnary(expr)

            is Expr.Match -> {
                val tScrut = checkExpr(expr.scrutinee)
                var resultType: Type? = null
                for (c in expr.cases) {
                    enterScope()
                    checkPattern(c.pattern, tScrut)
                    val tRes = checkExpr(c.result)
                    exitScope()
                    if (resultType == null) resultType = tRes
                    else if (tRes != resultType) {
                        throw TypeError("All match cases must return same type, got $resultType and $tRes", 0, 0)
                    }
                }
                resultType ?: throw TypeError("Match must have at least one case", 0, 0)
            }

            is Expr.BlockExpr -> {
                // 新作用域
                enterScope()
                var lastType: Type = Type.Void
                for (stmt in expr.statements) {
                    when (stmt) {
                        // 如果是最后一条并且是表达式语句，则拿它的类型作为结果
                        is Statement.ExprStmt -> {
                            lastType = checkExpr(stmt.expr)
                        }

                        is Statement.Return -> {
                            // return 在块表达式里直接当成结果
                            lastType = stmt.expr?.let { checkExpr(it) } ?: Type.Void
                            break
                        }

                        else -> {
                            checkStatement(stmt)
                            lastType = Type.Void
                        }
                    }
                }
                exitScope()
                lastType
            }

            else -> throw TypeError("Unsupported expression: ${expr::class.simpleName}", 0, 0)
        }
    }

    // --- 类型辅助 ---
    private fun canCast(src: Type, dest: Type): Boolean {
        if (src == dest) return true
        if (src == Type.Int && dest == Type.Float) return true
        if (dest is Type.Optional && src == dest.baseType) return true
        return false
    }

    private fun checkTypeAnnotation(t: Type): Type {
        when (t) {
            is Type.Generic -> t.typeArgs.forEach { checkTypeAnnotation(it) }
            is Type.Array -> checkTypeAnnotation(t.elementType)
            is Type.Optional -> checkTypeAnnotation(t.baseType)
            is Type.Future -> checkTypeAnnotation(t.baseType)
            is Type.TypeVar -> if (t.name !in currentTypeParams) {
                throw TypeError("Unknown type variable '${t.name}'", 0, 0)
            }

            is Type.Custom -> if (!interfaces.containsKey(t.name) && !records.containsKey(t.name)) {
                throw NameError("Unknown type '${t.name}'", 0, 0)
            }

            else -> {}
        }
        return t
    }

    private fun checkBinary(e: Expr.Binary): Type {
        val lt = checkExpr(e.left);
        val rt = checkExpr(e.right)
        return when (e.op) {
            ADD, SUB, MUL, DIV, MOD -> if (lt == rt && (lt == Type.Int || lt == Type.Float)) lt
            else throw TypeError("${e.op} requires Int/Float, got $lt and $rt", 0, 0)

            EQEQ, NEQ, LT, LE, GT, GE -> Type.Bool
            AND, OR -> if (lt == Type.Bool && rt == Type.Bool) Type.Bool
            else throw TypeError("${e.op} requires Bool, got $lt and $rt", 0, 0)
        }
    }

    private fun checkUnary(e: Expr.Unary): Type {
        val t = checkExpr(e.expr)
        return when (e.op) {
            NOT -> if (t == Type.Bool) Type.Bool else throw TypeError("! requires Bool, got $t", 0, 0)
            NEG -> if (t == Type.Int) Type.Int else throw TypeError("- requires Int, got $t", 0, 0)
        }
    }

    /** 检查模式是否能匹配给定的 scrutineeType，并在当前作用域绑定变量 */
    private fun checkPattern(pat: Pattern, scrutineeType: Type) {
        when (pat) {
            is Pattern.Wildcard -> return
            is Pattern.Literal -> {
                val litType = when (pat.value) {
                    is Long -> Type.Int
                    is Double -> Type.Float
                    is Boolean -> Type.Bool
                    is Char -> Type.Char
                    is String -> Type.String
                    else -> throw TypeError("Unsupported literal in pattern: ${pat.value}", 0, 0)
                }
                if (litType != scrutineeType) {
                    throw TypeError("Pattern literal type $litType does not match scrutinee type $scrutineeType", 0, 0)
                }
            }

            is Pattern.Var -> {
                // 变量模式，总是能匹配，绑定到 scrutineeType
                declareVar(pat.name, scrutineeType)
            }

            is Pattern.Constructor -> {
                // 构造器模式：只能对记录或自定义类型做解构
                if (scrutineeType !is Type.Custom) {
                    throw TypeError("Cannot match constructor ${pat.name} on non-custom type $scrutineeType", 0, 0)
                }
                val rec = records[scrutineeType.name]
                    ?: throw NameError("Unknown record type '${scrutineeType.name}'", 0, 0)
                if (rec.fields.size != pat.args.size) {
                    throw TypeError(
                        "Constructor '${pat.name}' expects ${rec.fields.size} args, got ${pat.args.size}",
                        0,
                        0
                    )
                }
                // 对每个子模式，用字段类型递归检查
                rec.fields.zip(pat.args).forEach { (field, subPat) ->
                    checkPattern(subPat, field.type)
                }
            }
        }
    }

    // --- 作用域 & 变量 ---
    private fun enterScope() {
        varScopes.add(mutableMapOf())
    }

    private fun exitScope() {
        varScopes.removeAt(varScopes.lastIndex)
    }

    private fun declareVar(name: String, type: Type) {
        if (varScopes.isEmpty()) varScopes.add(mutableMapOf())
        val scope = varScopes.last()
        if (scope.containsKey(name)) {
            throw DuplicateDeclarationError("Variable '$name' already declared", 0, 0)
        }
        scope[name] = type
    }

    private fun resolveVar(name: String): Type {
        for (i in varScopes.indices.reversed()) {
            varScopes[i][name]?.let { return it }
        }

        constants[name]?.let { return it }

        functions[name]?.let { fn ->
            val paramTs = fn.params.map { it.type }
            val retT = fn.returnType
                ?: throw FunctionCallError("Function '${fn.name}' must declare return type", 0, 0)
            return Type.Function(fn.typeParams, paramTs, retT, fn.isAsync)
        }

        imports[name]?.let { moduleName ->
            return Type.Module(moduleName)
        }
        throw NameError("Undefined variable or function '$name'", 0, 0)
    }

    // --- 泛型替换 ---
    private fun substitute(type: Type, env: Map<String, Type>): Type = when (type) {
        is Type.TypeVar -> env[type.name] ?: type
        is Type.Generic -> Type.Generic(type.baseName, type.typeArgs.map { substitute(it, env) })
        is Type.Array -> Type.Array(substitute(type.elementType, env))
        is Type.Optional -> Type.Optional(substitute(type.baseType, env))
        is Type.Future -> Type.Future(substitute(type.baseType, env))
        else -> type
    }
}