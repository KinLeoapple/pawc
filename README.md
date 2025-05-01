# 🐾 PawScript Beginner Guide

Welcome to **PawScript**—a “cute yet practical” statically-typed, functional-style scripting language. This README covers all syntax through v0.1, including the three new features: **type casts**, **exception handling**, and **module import**.

---

## Table of Contents

1. [Installation & Run](#installation--run)  
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
13. [Module Import](#module-import)  
14. [Complete Example](#complete-example)  

---

## 1. Installation & Run

1. Clone & build:
   ```bash
   git clone https://github.com/KinLeoapple/pawc.git
   cd pawc
   cargo build --release
   ```
2. Run a script:
   ```bash
   target/release/pawc hello.paw
   ```

---

## 2. Basic Structure

A PawScript program is a sequence of statements and function declarations. Execution starts at the top and runs each statement in order.

---

## 3. Data Types

- **Primitive**:
   - Integer types: `Int`, `Long`
   - Floating-point types: `Float`, `Double`
   - Others: `Bool`, `Char`, `String`
- **Generic**: `Array<T>`
- **Special**: `Any` (dynamic)

---

## 4. Variable Declaration

```paw
let x: Int = 10
x = x + 1           # reassignment
```

---

## 5. Expressions

- Arithmetic: `+`, `-`, `*`, `/`, `%`
- Comparison: `==`, `!=`, `<`, `<=`, `>`, `>=`
- Logical: `&&`, `||`, `!`
- String concat: `"Hi " + name + "!"`
- Grouping: `(a + b) * c`

---

## 6. Statements

- Declaration / assignment: `let` / `=`
- Output: `say <expr>`
- Input: `ask <"prompt">` or `let x: String <- ask "?"`
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
fun name(a: Int, b: Float): String {
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
let f: Float = i as Float   # Int → Float
say f + 1.5
```

- Numeric ↔ numeric (Int, Long, Float, Double)
- Casting to same type is no-op
- Invalid casts (e.g. String→Int) are compile errors

---

## 11. Comments

```paw
# single-line comment
let x: Int = 5   # trailing comment
```

---

## 12. Exception Handling

| Keyword   | Role                           |
|----------|--------------------------------|
| `bark`    | throw                          |
| `sniff`   | try                            |
| `snatch`  | catch (binds exception name)   |
| `lastly`  | finally                        |

### Throw

```paw
bark "error message"
```

Immediately jumps to nearest `snatch` block.

### Try-Catch-Finally

```paw
sniff {
  …            # try block
} snatch (e) {
  say "Caught: " + e
} lastly {
  say "Cleanup"
}
```

---

## 13. Module Import

PawScript can import other `.paw` scripts as modules, with optional aliases.

### Syntax

```paw
import foo.bar.baz           # imports foo/bar/baz.paw, alias “baz”
import utils.math as m       # imports utils/math.paw, alias “m”
```

- **module**: dot-separated identifiers → path `module.join("/") + ".paw"`
- Optional `as alias`: access prefix; defaults to last path segment

### Accessing Members

- Use property or call syntax on the alias:
  ```paw
  say "square(5) = " + m.square(5)
  say "PI = " + utils.math.PI    # if no alias used
  ```
- All top-level functions and variables of the imported script become members of that module namespace.

---

## 14. Complete Example

```paw
# import modules
import utils.math as m
import string

# Module tests
say "=== Module tests ==="
say "square(5) = " + m.square(5)
say "cube(3)   = " + m.cube(3)

# Array & indexing tests
say "\n=== Array & indexing tests ==="
let a: Array<Int> = [10,20,30,40]
say "a[0] = " + a[0]
say "a[2] = " + a[2]
say "a.length = " + a.length

# String module tests
say "\n=== String module tests ==="
let name: String = "PawScript"
say "length(name) = " + string.length(name)
say string.shout(name)

# Exception & type casting tests
fun reciprocal(x: Int): Float {
  if x == 0 {
    bark "division by zero"
  }
  return 1.0 / x
}

sniff {
  say "reciprocal(2) = " + reciprocal(2)
  say "reciprocal(0) = " + reciprocal(0)
} snatch (err) {
  say "Caught error: " + err
} lastly {
  say "Done exception test"
}

# Type casting
let i: Int = 7
say "i as Float = " + (i as Float)
say "i as Double = " + (i as Double)
```

---

Happy scripting!