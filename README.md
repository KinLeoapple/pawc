# 🐾 PawScript Quickstart Guide

Welcome to **PawScript** — a **cute yet practical** statically typed, functional scripting language. This README covers all of v0.1’s syntax and introduces three major features: **type casting**, **exception handling**, and **module import**. It also documents **optional types** and **null values**.

---

## Table of Contents

1. [Installation & Execution](#installation--execution)
2. [Program Structure](#program-structure)
3. [Data Types](#data-types)
4. [Optional Types & Null Values](#optional-types--null-values)
5. [Variable Declaration](#variable-declaration)
6. [Expressions](#expressions)
7. [Statements](#statements)
8. [Control Flow](#control-flow)
9. [Functions](#functions)
10. [Arrays](#arrays)
11. [Type Casting](#type-casting)
12. [Comments](#comments)
13. [Exception Handling](#exception-handling)
14. [Module Import](#module-import)
15. [Full Example](#full-example)

---

## Installation & Execution

1. Clone and build:

   ```bash
   git clone https://github.com/KinLeoapple/pawc.git
   cd pawc
   cargo build --release
   ```
2. Run a `.paw` script:

   ```bash
   target/release/pawc hello.paw
   ```

---

## Program Structure

A PawScript program consists of statements and function declarations, executed in order.

---

## Data Types

* **Primitive Types**: `Int`, `Long`, `Float`, `Double`, `Bool`, `Char`, `String`
* **Generic**: `Array<T>`
* **Special**:

    * `Any` — dynamic type
    * `Optional<T>` (or `T?`) — nullable type

---

## Optional Types & Null Values

PawScript supports nullable (optional) types to represent the absence of a value.

* Declare an optional type by appending `?`, e.g. `Int?` equals `Optional<Int>`.
* The null literal is `nopaw`, corresponding to runtime `null`.
* You can assign `nopaw` to optional variables only:

  ```paw
  let maybeNum: Int? = nopaw
  if maybeNum == nopaw {
    say "No number provided"
  }
  ```
* You must check for `nopaw` before unwrapping. Assigning `nopaw` to non-optional types is a compile-time error.

---

## Variable Declaration

```paw
let x: Int = 10
let y: Int? = nopaw    # optional type
x = x + 1             # reassignment
```

---

## Expressions

* Arithmetic: `+ - * / %`
* Comparison: `== != < <= > >=`
* Logical: `&& || !`
* String concatenation: `"Hi " + name + "!"`
* Grouping: `(a + b) * c`
* Optional comparisons: compare with `nopaw`

---

## Statements

* Declaration / assignment: `let` / `=`
* Output: `say <expr>`
* Input: `ask <"prompt">` or `let x: String <- ask "?"`
* Return: `return <expr>` or `return`

---

## Control Flow

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

## Functions

```paw
fun name(a: Int, b: Float): String {
  return "…"
}
let s: String = name(1, 2.5)
```

---

## Arrays

```paw
let a: Array<Int> = [1,2,3]
say a[0]        # index access
say a.length    # property
```

---

## Type Casting

Use `as` for casts:

```paw
let i: Int = 3
let f: Float = i as Float   # Int → Float
say f + 1.5
```

* Supported between Int/Long/Float/Double
* No-op when source and target match
* Invalid casts are compile-time errors

---

## Comments

```paw
# single-line comment
let x: Int = 5   # trailing comment
```

---

## Exception Handling

| Keyword  | Purpose       |
| -------- | ------------- |
| `bark`   | throw         |
| `sniff`  | try block     |
| `snatch` | catch block   |
| `lastly` | finally block |

### Throwing

```paw
bark "error message"
```

### Try-Catch-Finally

```paw
sniff {
  …
} snatch (e) {
  say "Caught: " + e
} lastly {
  say "Cleanup"
}
```

---

## Module Import

Import `.paw` files by path, with optional alias.

```paw
import utils.math       # alias defaults to "math"
import utils.math as m  # alias "m"
```

* Access members: `m.square(5)` or `utils.math.PI`

---

## Full Example

```paw
import utils.math as m
import string

say "=== Module tests ==="
say "square(5) = " + m.square(5)
say "cube(3)   = " + m.cube(3)

say "\n=== Array & indexing tests ==="
let a: Array<Int> = [10,20,30,40]
say "a[0] = " + a[0]
say "a.length = " + a.length

say "\n=== Nullable & nopaw tests ==="
let maybe: Int? = nopaw
if maybe == nopaw {
  say "maybe is null"
} else {
  say "maybe value = " + maybe
}

say "\n=== String module tests ==="
let name: String = "PawScript"
say "length(name) = " + string.length(name)
say string.shout(name)

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

let i: Int = 7
say "i as Float = " + (i as Float)
say "i as Double = " + (i as Double)
```

Happy coding with PawScript!
