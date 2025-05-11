package org.pawscript.semantic

import org.pawscript.ast.Declaration
import org.pawscript.ast.Type

data class ModuleContents(
    val functions: List<Declaration.Fun>,
    val constants: List<Pair<String, Type>>
)