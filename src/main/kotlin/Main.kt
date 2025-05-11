package org.pawscript

import org.pawscript.error.PawError
import org.pawscript.lexer.Lexer
import org.pawscript.parser.Parser
import org.pawscript.semantic.TypeChecker
import java.io.File

fun main(args: Array<String>) {
    if (args.size != 1) {
        println("Usage: pawc <script.paw>")
        return
    }
    val path = args[0]
    try {
        // 1. 读源码
        val source = File(path).readText()

        // 2. 词法分析
        val tokens = Lexer(source).tokenize()

        // 3. 语法解析
        val nodes = Parser(tokens).parse()

        // 4. 语义检查
        TypeChecker().check(nodes)

        // 5. 运行
//        Interpreter().execute(nodes)

    } catch (e: PawError) {
        // PawError 包含行列、提示等
        println(e)
    } catch (e: Exception) {
        // 其他未捕获异常
        e.printStackTrace()
    }
}