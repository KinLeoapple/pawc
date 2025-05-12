package org.pawscript.interpreter

sealed class Thunk {
    /**
     * Evaluate an expression in the given environment, then pass the result to cont to get next Thunk.
     */
    data class EvalExpr(
        val expr: org.pawscript.ast.Expr,
        val env: Env,
        val cont: (Value) -> Thunk
    ) : Thunk()

    /**
     * Execute a statement in the given environment, then call cont for next Thunk.
     */
    data class ExecStmt(
        val stmt: org.pawscript.ast.Statement,
        val env: Env,
        val cont: () -> Thunk
    ) : Thunk()

    /**
     * Signal a return from function, carrying the return Value.
     */
    data class Return(val value: Value) : Thunk()
}

fun trampoline(start: Thunk): Value {
    var currentThunk = start
    while (true) {
        currentThunk = when (val thunk = currentThunk) {
            is Thunk.EvalExpr -> evalExprStep(thunk)
            is Thunk.ExecStmt -> execStmtStep(thunk)
            is Thunk.Return   -> return thunk.value
        }
    }
}

/**
 * Perform one evaluation step for an expression Thunk.
 */
private fun evalExprStep(thunk: Thunk.EvalExpr): Thunk {
    // TODO: implement expression evaluation logic (literals, variables, calls, etc.)
    error("evalExprStep not implemented for expr=${thunk.expr}")
}

/**
 * Perform one execution step for a statement Thunk.
 */
private fun execStmtStep(thunk: Thunk.ExecStmt): Thunk {
    // TODO: implement statement execution logic (if, loop, return, I/O, etc.)
    error("execStmtStep not implemented for stmt=${thunk.stmt}")
}