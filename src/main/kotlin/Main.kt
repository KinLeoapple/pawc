package org.pawscript

import org.pawscript.config.ConfigLoader
import org.pawscript.error.PawError
import org.pawscript.lexer.Lexer
import org.pawscript.parser.Parser
import org.pawscript.semantic.ModuleContents
import org.pawscript.semantic.TypeChecker
import org.pawscript.ast.Declaration
import org.pawscript.ast.Statement
import java.nio.file.Files
import java.nio.file.Path
import java.nio.file.Paths

fun main(args: Array<String>) {
    if (args.size != 1) {
        println("Usage: pawc <script.paw>")
        return
    }

    val scriptPath = Paths.get(args[0]).toAbsolutePath().normalize()
    if (!Files.exists(scriptPath)) {
        println("Error: script file not found: $scriptPath")
        return
    }

    val projectRoot = scriptPath.ancestors()
        .firstOrNull { it.fileName.toString() == "src" }
        ?.parent
        ?: scriptPath.parent  // 否则退回到脚本同级目录

    val cfg = ConfigLoader.load(projectRoot)
    val (srcDir, stdDir) = ConfigLoader.resolvePaths(projectRoot, cfg)

    val externalModules = mutableMapOf<String, ModuleContents>()
    externalModules += loadAllModules(stdDir)
    externalModules += loadAllModules(srcDir)

    try {
        val source = Files.readString(scriptPath)
        val tokens = Lexer(source).tokenize()
        val nodes  = Parser(tokens).parse()

        val checker = TypeChecker(externalModules)
        checker.check(nodes)

        println("Compilation and type checking succeeded!")

        // Interpreter(externalModules).run(nodes)

    } catch (e: PawError) {
        println("Error: $e")
    }
}

/**
 * 扫描给定目录下的所有 .paw 文件，解析其 AST 并提取顶层函数和常量，
 * 汇总成 ModuleContents。已加载的模块名不会重复。
 */
fun loadAllModules(dir: Path): Map<String, ModuleContents> {
    val result = mutableMapOf<String, ModuleContents>()
    if (!Files.isDirectory(dir)) return result

    Files.newDirectoryStream(dir, "*.paw").use { stream ->
        for (path in stream) {
            val name = path.fileName.toString().removeSuffix(".paw")
            if (result.containsKey(name)) continue

            val src   = Files.readString(path)
            val nodes = Parser(Lexer(src).tokenize()).parse()

            val funcs = nodes.filterIsInstance<Declaration.Fun>()
            val consts = nodes.filterIsInstance<Statement.Let>()
                .map { it.name to (it.type ?: error("Constant '${it.name}' must have a type")) }

            result[name] = ModuleContents(functions = funcs, constants = consts)
        }
    }
    return result
}


fun Path.ancestors(): Sequence<Path> = sequence {
    var p: Path? = this@ancestors
    while (p != null) {
        yield(p)
        p = p.parent
    }
}