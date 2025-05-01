# 🐾 PawScript Beginner’s Guide

Welcome to **PawScript**—a cute yet practical, statically-typed, functional-style scripting language. This README covers all syntax through v0.1, plus new **Type Casting** and **Exception Handling** sections.

---

## Table of Contents

1. [Installation & Running](#installation--running)
2. [Basic Structure](#basic-structure)
3. [Data Types](#data-types)
4. [Variable Declaration](#variable-declaration)
5. [Expressions](#expressions)
6. [Statements](#statements)
7. [Control Flow](#control-flow)
8. [Functions](#functions)
9. [Arrays](#arrays)
10. [Type Casting](#type-casting)
11. [Comments](#comments)
12. [Exception Handling](#exception-handling)
13. [Complete Example](#complete-example)

---

## 1. Installation & Running

1. **Clone & build**:
   ```bash
   git clone https://github.com/KinLeoapple/pawc.git
   cd pawc
   cargo build --release
   ```
2. **Interpret**:
   ```bash
   target/release/pawc run hello.paw
   ```

---

## 2. Basic Structure

A script is a series of statements or function declarations. Execution starts at the top and runs sequentially.

---

## 3. Data Types

- **Primitive**: `Int`, `Float`, `Bool`, `Char`, `String`
- **Generic**: `Array<T>`
- **Dynamic**: `Any`

---

## 4. Variable Declaration

```paw
let x: Int = 10       # immutable binding
x = x + 1             # assignment (variable)
```

---

## 5. Expressions

- Arithmetic: `+` `-` `*` `/` `%`
- Comparison: `==` `!=` `<` `<=` `>` `>=`
- Logical: `&&` `||` `!`
- String concat: `"Hi " + name + "!"`
- Grouping: `(a + b) * c`

---

## 6. Statements

- Declaration/assignment: `let` / then `=`
- Print: `say <expr>`
- Input: `ask <string>` or `let x: String <- ask "?"`
- Return: `return <expr>` or `return`

---

## 7. Control Flow

```paw
if cond {
  …
} else if cond2 {
  …
} else {
  …
}

loop forever { … }
loop cond { … }
loop i in start..end { … }
```

---

## 8. Functions

```paw
fun name(param1: Int, param2: Float): String {
  return "…"
}
let s: String = name(1, 2.5)
```

---

## 9. Arrays

```paw
let a: Array<Int> = [1,2,3]
say a[0]        # index
say a.length    # property
```

---

## 10. Type Casting

Use `as` for explicit casts:

```paw
let i: Int = 3
let f: Float = i as Float     # Int → Float
say f + 1.5
```

- Numeric widening (Int⇄Float)
- Same-type casts are no-ops
- Incompatible casts (String→Int) are compile-time errors

---

## 11. Comments

```paw
# single-line comment
let x: Int = 5   # end-of-line comment
```

---

## 12. Exception Handling

Keywords:

| Keyword   | Role                             |
|-----------|----------------------------------|
| `bark`    | throw an exception               |
| `sniff`   | begin try block                  |
| `snatch`  | catch block (binds exception var)|
| `lastly`  | finally block (always executes)  |

### Throw

```paw
bark "error message"
```

Immediately jumps to nearest `snatch`.

### Catch

```paw
sniff {
  …        # try block
} snatch (e) {
  …        # catch block, e is the error message
} lastly {
  …        # finally block
}
```

- If no `bark`, `snatch` is skipped; `lastly` still runs
- If `bark`, executes `snatch` then `lastly`

---

## 13. Complete Example

```paw
# reciprocal throws on zero, uses cast
fun reciprocal(x: Int): Float {
    if x == 0 {
        bark "division by zero"      # throw
    }
    return 1.0 / (x as Float)        # cast
}

sniff {
    say "Calling reciprocal(2)…"
    let a: Float = reciprocal(2)
    say "Result: " + a

    say "Calling reciprocal(0)…"
    let b: Float = reciprocal(0)    # throws → jumps to snatch
    say "Won’t run"
} snatch (err) {
    say "Caught error: " + err       # catch
} lastly {
    say "Cleanup done"
}

say "Done."
```

**Expected output:**
```
Calling reciprocal(2)…
Result: 0.5
Calling reciprocal(0)…
Caught error: division by zero
Cleanup done
Done.
```  

Enjoy PawScript! For bootstrapping and extension, see the source.