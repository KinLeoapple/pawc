package org.pawscript.parser

import org.pawscript.ast.BinaryOp
import org.pawscript.ast.Declaration
import org.pawscript.ast.Expr
import org.pawscript.ast.MethodSig
import org.pawscript.ast.Node
import org.pawscript.ast.Param
import org.pawscript.ast.Pattern
import org.pawscript.ast.Statement
import org.pawscript.ast.TemplatePart
import org.pawscript.ast.Type
import org.pawscript.ast.UnaryOp
import org.pawscript.error.PawError.ParseError
import org.pawscript.lexer.Token
import org.pawscript.lexer.TokenKind

/**
 * Recursive-descent parser for PawScript.
 * Converts a list of tokens into an AST (list of Statement).
 */
class Parser(private val tokens: List<Token>) {
    private var current = 0
    private val typeParamStack = mutableListOf<List<String>>()
    private val currentTypeParams: List<String>
        get() = typeParamStack.lastOrNull() ?: emptyList()

    /** 解析入口，返回混合 Declaration 和 Statement 的节点列表 */
    @Throws(ParseError::class)
    fun parse(): List<Node> {
        val nodes = mutableListOf<Node>()
        while (!isAtEnd()) {
            nodes += parseTopLevel()
        }
        return nodes
    }

    /** 解析一个顶层声明或语句 */
    @Throws(ParseError::class)
    private fun parseTopLevel(): Node {
        return when {
            match(TokenKind.IMPORT) -> parseImport()
            match(TokenKind.TAIL) -> parseTail()
            match(TokenKind.RECORD) -> parseRecord()
            match(TokenKind.ASYNC) && match(TokenKind.FUN) -> parseFun(isAsync = true)
            match(TokenKind.FUN) -> parseFun(isAsync = false)
            else -> parseStatement()
        }
    }

    // --- Declaration Parsers ---

    /** import module [as alias] */
    @Throws(ParseError::class)
    private fun parseImport(): Declaration.Import {
        val module = consume(TokenKind.IDENTIFIER, "Expected module name after 'import'").lexeme
        val alias = if (match(TokenKind.AS)) {
            consume(TokenKind.IDENTIFIER, "Expected alias name after 'as'").lexeme
        } else {
            null
        }
        return Declaration.Import(module, alias)
    }

    /** tail Name { methods... } */
    @Throws(ParseError::class)
    private fun parseTail(): Declaration.Tail {
        val interfaceName = consume(TokenKind.IDENTIFIER, "Expected interface name after 'tail'").lexeme
        consume(TokenKind.LBRACE, "Expected '{' after interface name")
        val methods = mutableListOf<MethodSig>()
        while (!check(TokenKind.RBRACE) && !isAtEnd()) {
            val mName = consume(TokenKind.IDENTIFIER, "Expected method name").lexeme
            consume(TokenKind.LPAREN, "Expected '(' after method name")
            val params = mutableListOf<Param>()
            if (!check(TokenKind.RPAREN)) {
                do {
                    val pName = consume(TokenKind.IDENTIFIER, "Expected parameter name").lexeme
                    if (pName == "self") {
                        if (match(TokenKind.COLON)) parseType()
                        params += Param("self", Type.Custom(interfaceName))
                    } else {
                        consume(TokenKind.COLON, "Expected ':' after parameter name")
                        val pType = parseType()
                        params += Param(pName, pType)
                    }
                } while (match(TokenKind.COMMA))
            }
            consume(TokenKind.RPAREN, "Expected ')' after parameters")
            consume(TokenKind.COLON, "Expected ':' before return type")
            val retType = parseType()
            methods += MethodSig(mName, params, retType)
        }
        consume(TokenKind.RBRACE, "Expected '}' after interface body")
        return Declaration.Tail(interfaceName, methods)
    }

    /** fun name(params): Type? { body } */
    @Throws(ParseError::class)
    private fun parseFun(isAsync: Boolean): Declaration.Fun {
        // 1. 解析泛型声明 <T, U, ...>
        val typeParams = mutableListOf<String>()
        if (match(TokenKind.LT)) {
            do {
                typeParams += consume(TokenKind.IDENTIFIER, "Expected type parameter name").lexeme
            } while (match(TokenKind.COMMA))
            consume(TokenKind.GT, "Expected '>' after type parameters")
        }

        // —— **压栈** ——
        typeParamStack.add(typeParams)

        // 2. 解析可选接收者 (ReceiverType)
        var receiverType: String? = null
        if (match(TokenKind.LPAREN) && check(TokenKind.IDENTIFIER) && peekNext().kind == TokenKind.RPAREN) {
            receiverType = advance().lexeme
            consume(TokenKind.RPAREN, "Expected ')' after receiver type")
        }

        // 3. 解析函数名
        val name = consume(TokenKind.IDENTIFIER, "Expected function name after 'fun'").lexeme

        // 4. 解析形参列表
        consume(TokenKind.LPAREN, "Expected '(' after function name")
        val params = mutableListOf<Param>()
        if (!check(TokenKind.RPAREN)) {
            do {
                val pName = consume(TokenKind.IDENTIFIER, "Expected parameter name").lexeme
                if (pName == "self" && receiverType != null) {
                    if (match(TokenKind.COLON)) parseType()  // 丢弃类型注解
                    params += Param("self", Type.Custom(receiverType))
                } else {
                    consume(TokenKind.COLON, "Expected ':' after parameter name")
                    val pType = parseType()
                    params += Param(pName, pType)
                }
            } while (match(TokenKind.COMMA))
        }
        consume(TokenKind.RPAREN, "Expected ')' after parameters")

        // 5. 解析返回类型（可选）
        val returnType = if (match(TokenKind.COLON)) {
            parseType()
        } else {
            null
        }

        // 6. 解析函数体
        val body = parseBlock()

        // —— **出栈** ——
        typeParamStack.removeAt(typeParamStack.lastIndex)

        // 7. 构造 AST
        return Declaration.Fun(
            typeParams = typeParams,
            receiverType = receiverType,
            name = name,
            params = params,
            returnType = returnType,
            body = body,
            isAsync = isAsync
        )
    }

    /** record Name { fields... } */
    @Throws(ParseError::class)
    private fun parseRecord(): Declaration.Record {
        val name = consume(TokenKind.IDENTIFIER, "Expected record name").lexeme

        // 解析 implements 列表
        val impls = if (match(TokenKind.LPAREN)) {
            val list = mutableListOf<String>()
            do {
                val iface = consume(TokenKind.IDENTIFIER, "Expected interface name").lexeme
                list += iface
            } while (match(TokenKind.COMMA))
            consume(TokenKind.RPAREN, "Expected ')' after interface list")
            list
        } else {
            emptyList()
        }

        consume(TokenKind.LBRACE, "Expected '{' after record header")
        val fields = mutableListOf<Param>()
        while (!check(TokenKind.RBRACE) && !isAtEnd()) {
            val fName = consume(TokenKind.IDENTIFIER, "Expected field name").lexeme
            consume(TokenKind.COLON, "Expected ':' after field name")
            val fType = parseType()
            match(TokenKind.COMMA)
            fields += Param(fName, fType)
        }
        consume(TokenKind.RBRACE, "Expected '}' after record body")
        return Declaration.Record(name, impls, fields)
    }

    // --- Statement Parsers ---

    @Throws(ParseError::class)
    private fun parseStatement(): Statement {
        // Assign: identifier = expr
        if (check(TokenKind.IDENTIFIER) && peekNext().kind == TokenKind.ASSIGN) {
            val name = advance().lexeme
            advance() // '='
            val expr = parseExpression()
            match(TokenKind.SEMICOLON)
            return Statement.Assign(name, expr)
        }
        return when {
            match(TokenKind.LET) -> parseLet()
            match(TokenKind.IF) -> parseIf()
            match(TokenKind.LOOP) -> parseLoop()
            match(TokenKind.RETURN) -> parseReturn()
            match(TokenKind.BARK) -> parseBark()
            match(TokenKind.SNIFF) -> parseSniff()
            match(TokenKind.SAY) -> parseSay()
            match(TokenKind.ASK) -> parseAsk()
            match(TokenKind.BREAK) -> Statement.Break
            match(TokenKind.CONTINUE) -> Statement.Continue
            else -> parseExpressionStatement()
        }
    }

    private fun parseSay(): Statement {
        // say expr
        val expr = parseExpression()
        return Statement.Say(expr)
    }

    private fun parseAsk(): Statement {
        // ask "prompt"
        val promptToken = consume(TokenKind.STRING_LITERAL, "Expected string prompt in ask")
        return Statement.Ask(promptToken.lexeme)
    }

    @Throws(ParseError::class)
    private fun parseLet(): Statement.Let {
        val name = consume(TokenKind.IDENTIFIER, "Expected variable name").lexeme
        val type = if (match(TokenKind.COLON)) parseType() else null
        consume(TokenKind.ASSIGN, "Expected '=' after variable declaration")
        val expr = parseExpression()
        return Statement.Let(name, type, expr)
    }

    @Throws(ParseError::class)
    private fun parseIf(): Statement.If {
        val condition = parseExpression()
        val thenBranch = parseBlock()
        val elseBranch = if (match(TokenKind.ELSE)) {
            if (match(TokenKind.IF)) listOf(parseIf()) else parseBlock()
        } else null
        return Statement.If(condition, thenBranch, elseBranch)
    }

    @Throws(ParseError::class)
    private fun parseLoop(): Statement {
        if (check(TokenKind.LBRACE)) return Statement.LoopInfinite(parseBlock())

        // indexed loop
        if (check(TokenKind.IDENTIFIER) && peekNext().kind == TokenKind.COMMA) {
            val idx = advance().lexeme
            advance() // comma
            val itm = consume(TokenKind.IDENTIFIER, "Expected item name").lexeme
            consume(TokenKind.IN, "Expected 'in'")
            val arr = parseExpression()
            val b = parseBlock()
            return Statement.LoopIndexed(idx, itm, arr, b)
        }

        // in or range loop
        if (check(TokenKind.IDENTIFIER) && peekNext().kind == TokenKind.IN) {
            val itm = advance().lexeme; advance()
            val first = parseExpression()
            if (match(TokenKind.RANGE)) {
                val second = parseExpression()
                val b = parseBlock()
                return Statement.LoopRange(itm, first, second, b)
            } else {
                val b = parseBlock()
                return Statement.LoopIn(itm, first, b)
            }
        }

        // while
        val cond = parseExpression()
        val b = parseBlock()
        return Statement.LoopWhile(cond, b)
    }

    @Throws(ParseError::class)
    private fun parseReturn(): Statement.Return {
        val expr = if (!check(TokenKind.RBRACE) && !isAtEnd()) parseExpression() else null
        return Statement.Return(expr)
    }

    @Throws(ParseError::class)
    private fun parseBark(): Statement.Bark {
        val expr = parseExpression()
        return Statement.Bark(expr)
    }

    @Throws(ParseError::class)
    private fun parseSniff(): Statement.Sniff {
        val tryB = parseBlock()
        var catchV: String? = null
        var catchB: List<Statement>? = null
        if (match(TokenKind.SNATCH)) {
            consume(TokenKind.LPAREN, "Expected '(' after snatch")
            catchV = consume(TokenKind.IDENTIFIER, "Expected exception var").lexeme
            consume(TokenKind.RPAREN, "Expected ')'")
            catchB = parseBlock()
        }
        var finB: List<Statement>? = null
        if (match(TokenKind.LASTLY)) finB = parseBlock()
        return Statement.Sniff(tryB, catchV, catchB, finB)
    }

    @Throws(ParseError::class)
    private fun parseExpressionStatement(): Statement.ExprStmt {
        val expr = parseExpression()
        match(TokenKind.SEMICOLON)
        return Statement.ExprStmt(expr)
    }

    // --- Expression Parsing ---

    private fun parseExpression(): Expr {
        return when {
            match(TokenKind.MATCH) -> parseMatch()
            else -> parseOr()
        }
    }

    private fun parseOr(): Expr {
        var e = parseAnd()
        while (match(TokenKind.OROR)) {
            e = Expr.Binary(e, BinaryOp.OR, parseAnd())
        }
        return e
    }

    @Throws(ParseError::class)
    private fun parseMatch(): Expr {
        // 已 match MATCH
        val scrutinee = parseExpression()
        consume(TokenKind.LBRACE, "Expected '{' after match scrutinee")

        val cases = mutableListOf<Expr.Case>()
        do {
            val pat = parsePattern()
            consume(TokenKind.ARROW, "Expected '->' after pattern")

            // 如果下一个是 LBRACE，则把分支体当成 BlockExpr
            val resultExpr = if (check(TokenKind.LBRACE)) {
                // parseBlock 返回 List<Statement>, 但我们需要 BlockExpr
                val stmts = parseBlock()
                Expr.BlockExpr(stmts)
            } else {
                // 单表达式分支
                parseExpression()
            }
            cases += Expr.Case(pat, resultExpr)
        } while (match(TokenKind.COMMA))

        consume(TokenKind.RBRACE, "Expected '}' after match cases")
        return Expr.Match(scrutinee, cases)
    }

    /** 解析 Pattern */
    @Throws(ParseError::class)
    private fun parsePattern(): Pattern {
        return when {
            match(TokenKind.UNDERSCORE) -> Pattern.Wildcard
            match(TokenKind.INT_LITERAL) -> Pattern.Literal(previous().lexeme.toLong())
            match(TokenKind.FLOAT_LITERAL) -> Pattern.Literal(previous().lexeme.toDouble())
            match(TokenKind.STRING_LITERAL) -> Pattern.Literal(previous().lexeme)
            match(TokenKind.CHAR_LITERAL) -> Pattern.Literal(previous().lexeme.first())
            match(TokenKind.IDENTIFIER) -> {
                val name = previous().lexeme
                if (match(TokenKind.LPAREN)) {
                    // 构造器模式：Name(p1,p2,…)
                    val args = mutableListOf<Pattern>()
                    if (!check(TokenKind.RPAREN)) {
                        do {
                            args += parsePattern()
                        } while (match(TokenKind.COMMA))
                    }
                    consume(TokenKind.RPAREN, "Expected ')' after constructor pattern")
                    Pattern.Constructor(name, args)
                } else {
                    // 绑定变量
                    Pattern.Var(name)
                }
            }
            else -> throw ParseError("Unexpected token in pattern: ${peek().kind}", peek().line, peek().col)
        }
    }

    private fun parseAnd(): Expr {
        var e = parseEquality()
        while (match(TokenKind.ANDAND)) {
            e = Expr.Binary(e, BinaryOp.AND, parseEquality())
        }
        return e
    }

    private fun parseEquality(): Expr {
        var e = parseComparison()
        while (true) {
            when {
                match(TokenKind.EQEQ) -> e = Expr.Binary(e, BinaryOp.EQEQ, parseComparison())
                match(TokenKind.NEQ) -> e = Expr.Binary(e, BinaryOp.NEQ, parseComparison())
                else -> break
            }
        }
        return e
    }

    private fun parseComparison(): Expr {
        var e = parseTerm()
        while (true) {
            when {
                match(TokenKind.LT) -> e = Expr.Binary(e, BinaryOp.LT, parseTerm())
                match(TokenKind.LE) -> e = Expr.Binary(e, BinaryOp.LE, parseTerm())
                match(TokenKind.GT) -> e = Expr.Binary(e, BinaryOp.GT, parseTerm())
                match(TokenKind.GE) -> e = Expr.Binary(e, BinaryOp.GE, parseTerm())
                else -> break
            }
        }
        return e
    }

    private fun parseTerm(): Expr {
        var e = parseFactor()
        while (true) {
            when {
                match(TokenKind.PLUS) -> e = Expr.Binary(e, BinaryOp.ADD, parseFactor())
                match(TokenKind.MINUS) -> e = Expr.Binary(e, BinaryOp.SUB, parseFactor())
                else -> break
            }
        }
        return e
    }

    private fun parseCast(): Expr {
        var expr = parseUnary()
        // allow chaining: expr as Type as Type2 …
        while (match(TokenKind.AS)) {
            val target = parseType()
            expr = Expr.As(expr, target)
        }
        return expr
    }

    private fun parseFactor(): Expr {
        var e = parseCast()
        while (true) {
            when {
                match(TokenKind.STAR) -> e = Expr.Binary(e, BinaryOp.MUL, parseUnary())
                match(TokenKind.SLASH) -> e = Expr.Binary(e, BinaryOp.DIV, parseUnary())
                match(TokenKind.PERCENT) -> e = Expr.Binary(e, BinaryOp.MOD, parseUnary())
                else -> break
            }
        }
        return e
    }

    private fun parseUnary(): Expr {
        if (match(TokenKind.AWAIT)) {
            // parse the operand as a sub-expression
            val inner = parseUnary()
            return Expr.Await(inner)
        }
        if (match(TokenKind.NOT)) return Expr.Unary(UnaryOp.NOT, parseUnary())
        if (match(TokenKind.MINUS)) return Expr.Unary(UnaryOp.NEG, parseUnary())
        return parsePostfix(parsePrimary())
    }

    @Throws(ParseError::class)
    private fun parsePrimary(): Expr {
        // 1) 基础表达式

        // 1.1 ask 表达式
        if (match(TokenKind.ASK)) {
            val prompt = consume(TokenKind.STRING_LITERAL, "Expected string prompt after 'ask'").lexeme
            return Expr.AskExpr(prompt)
        }

        // 1.2 数组字面量
        if (match(TokenKind.LBRACKET)) {
            val elements = mutableListOf<Expr>()
            if (!check(TokenKind.RBRACKET)) {
                do {
                    elements += parseExpression()
                } while (match(TokenKind.COMMA))
            }
            consume(TokenKind.RBRACKET, "Expected ']' after array literal")
            return Expr.ArrayLiteral(elements)
        }

        // 1.3 标识符（变量或字面量 true/false）
        if (match(TokenKind.IDENTIFIER)) {
            val name = previous().lexeme
            if (name == "true" || name == "false") {
                return Expr.LiteralBool(name.toBoolean())
            }
            // 先当作普通变量，后面在后缀循环里可能升级为 Call 或 MethodCall
            return Expr.Variable(name)
        }

        // 1.4 字符串、字符、数字、nopaw
        if (match(TokenKind.STRING_LITERAL)) return Expr.LiteralString(previous().lexeme)
        if (match(TokenKind.CHAR_LITERAL)) return Expr.LiteralChar(previous().lexeme.first())
        if (match(TokenKind.FLOAT_LITERAL)) return Expr.LiteralFloat(previous().lexeme.toDouble())
        if (match(TokenKind.INT_LITERAL)) return Expr.LiteralInt(previous().lexeme.toLong())
        if (match(TokenKind.NOPAW)) return Expr.Nopaw

        // 1.5 分组表达式
        if (match(TokenKind.LPAREN)) {
            val expr = parseExpression()
            consume(TokenKind.RPAREN, "Expected ')' after expression")
            return expr
        }

        // 1.6 字符串模板
        if (check(TokenKind.TEMPLATE_TEXT) || check(TokenKind.TEMPLATE_EXPR_START)) {
            val parts = mutableListOf<TemplatePart>()
            while (check(TokenKind.TEMPLATE_TEXT) || check(TokenKind.TEMPLATE_EXPR_START)) {
                if (match(TokenKind.TEMPLATE_TEXT)) {
                    parts += TemplatePart.Text(previous().lexeme)
                } else {
                    // TEMPLATE_EXPR_START
                    advance()  // 消费 `${`
                    val expr = parseExpression()
                    consume(TokenKind.TEMPLATE_EXPR_END, "Expected '}' after template expression")
                    parts += TemplatePart.ExprPart(expr)
                }
            }
            return Expr.StringTemplate(parts)
        }

        throw ParseError("Unexpected token '${peek().kind}'", peek().line, peek().col)
    }

    @Throws(ParseError::class)
    private fun parsePostfix(expr: Expr): Expr {
        var result = expr

        loop@ while (true) {
            // —— 1. 泛型函数调用：id<Int>(…) ——
            if (result is Expr.Variable && check(TokenKind.LT)) {
                // 1.1 读取所有类型实参
                advance() // consume '<'
                val typeArgs = mutableListOf<Type>()
                if (!check(TokenKind.GT)) {
                    do {
                        typeArgs += parseType()
                    } while (match(TokenKind.COMMA))
                }
                consume(TokenKind.GT, "Expected '>' after generic type arguments")
                // 1.2 构造临时 Call 节点，先不带位置/命名参数
                result = Expr.Call(
                    callee     = result.name,
                    typeArgs   = typeArgs,
                    positional = emptyList(),
                    named      = emptyMap()
                )
                continue@loop
            }

            // —— 2. 模块成员访问或调用：u.foo or u.foo(…) ——
            if (result is Expr.Variable && match(TokenKind.DOT)) {
                val alias  = result.name
                val member = consume(TokenKind.IDENTIFIER, "Expected member name after '$alias.'").lexeme

                if (match(TokenKind.LPAREN)) {
                    // 2.1 模块方法调用
                    val (positional, named) = parseArguments()
                    result = Expr.Call(
                        callee     = "$alias.$member",
                        typeArgs   = emptyList(),
                        positional = positional,
                        named      = named
                    )
                } else {
                    // 2.2 模块属性访问
                    result = Expr.ModuleAccess(alias, member)
                }
                continue@loop
            }

            when {
                // —— 3. 普通字段访问 .field ——
                match(TokenKind.DOT) -> {
                    val field = consume(TokenKind.IDENTIFIER, "Expected field name after '.'").lexeme
                    result = Expr.FieldAccess(result, field)
                }

                // —— 4. 括号调用 ——
                match(TokenKind.LPAREN) -> {
                    val (positional, named) = parseArguments()
                    result = when (result) {
                        // 4.1 方法调用：obj.method(...)
                        is Expr.FieldAccess ->
                            Expr.MethodCall(result.target, result.field, positional, named)

                        // 4.2 泛型或普通函数调用已经在上面被处理成 Expr.Call
                        is Expr.Call ->
                            // 先前若已读过 typeArgs，则这里把参数加上
                            result.copy(positional = positional, named = named)

                        // 4.3 普通顶层函数调用：foo(...)
                        is Expr.Variable ->
                            Expr.Call(result.name, emptyList(), positional, named)

                        else ->
                            throw ParseError(
                                "Cannot call expression of type ${result::class.simpleName}",
                                peek().line, peek().col
                            )
                    }
                }

                else -> break@loop
            }
        }

        return result
    }

    /** 解析一对圆括号中的位置参数和命名参数 */
    @Throws(ParseError::class)
    private fun parseArguments(): Pair<List<Expr>, Map<String,Expr>> {
        val positional = mutableListOf<Expr>()
        val named      = mutableMapOf<String,Expr>()
        if (!check(TokenKind.RPAREN)) {
            do {
                if (check(TokenKind.IDENTIFIER) && peekNext().kind == TokenKind.COLON) {
                    val name = advance().lexeme
                    advance() // consume ':'
                    named[name] = parseExpression()
                } else {
                    positional += parseExpression()
                }
            } while (match(TokenKind.COMMA))
        }
        consume(TokenKind.RPAREN, "Expected ')' after arguments")
        return positional to named
    }

    @Throws(ParseError::class)
    private fun parseType(): Type {
        // —— 1. 先读名字，不做任何提前 return ——
        val token = consume(TokenKind.IDENTIFIER, "Expected type name or type variable")
        val name = token.lexeme

        // —— 2. 决定 baseType，但不 return ——
        val baseType: Type = when {
            // 2.1 泛型实参 <...>
            match(TokenKind.LT) -> {
                val args = mutableListOf<Type>()
                if (!check(TokenKind.GT)) {
                    do {
                        args += parseType()
                    } while (match(TokenKind.COMMA))
                }
                consume(TokenKind.GT, "Expected '>' after generic arguments")
                // 特殊 Future<T>
                if (name == "Future" && args.size == 1) {
                    Type.Future(args[0])
                } else {
                    Type.Generic(name, args)
                }
            }
            // 2.2 类型变量也只是先赋值，不 return
            name in currentTypeParams -> Type.TypeVar(name)
            // 2.3 普通类型
            else -> when (name) {
                "Int" -> Type.Int
                "Long" -> Type.Long
                "Float" -> Type.Float
                "Double" -> Type.Double
                "Bool" -> Type.Bool
                "Char" -> Type.Char
                "String" -> Type.String
                "Void" -> Type.Void
                else -> Type.Custom(name)
            }
        }

        // —— 3. 数组后缀 [] ——
        val arrayed = if (match(TokenKind.LBRACKET) && match(TokenKind.RBRACKET)) {
            Type.Array(baseType)
        } else {
            baseType
        }

        // —— 4. 可空后缀 ? ——
        val nullable = if (match(TokenKind.QUESTION)) {
            Type.Optional(arrayed)
        } else {
            arrayed
        }

        // —— 5. 统一返回 ——
        return nullable
    }

    @Throws(ParseError::class)
    private fun parseBlock(): List<Statement> {
        consume(TokenKind.LBRACE, "Expected '{' before block")
        val stmts = mutableListOf<Statement>()
        while (!check(TokenKind.RBRACE) && !isAtEnd()) {
            stmts += parseStatement()
        }
        consume(TokenKind.RBRACE, "Expected '}' after block")
        return stmts
    }

    // --- 辅助方法 ---

    private fun match(vararg kinds: TokenKind): Boolean {
        for (k in kinds) if (check(k)) {
            advance(); return true
        }
        return false
    }

    private fun check(kind: TokenKind): Boolean = !isAtEnd() && peek().kind == kind
    private fun peekNext(): Token = tokens.getOrElse(current + 1) { tokens.last() }
    private fun advance(): Token {
        if (!isAtEnd()) current++; return previous()
    }

    private fun isAtEnd(): Boolean = peek().kind == TokenKind.EOF
    private fun peek(): Token = tokens[current]
    private fun previous(): Token = tokens[current - 1]

    @Throws(ParseError::class)
    private fun consume(kind: TokenKind, msg: String): Token {
        if (check(kind)) return advance()
        throw ParseError(msg, peek().line, peek().col)
    }
}