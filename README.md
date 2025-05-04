# 🐾 PawScript Getting Started Guide

Welcome to **PawScript** — a “cute yet practical” statically-typed, functional scripting language. This README covers all v0.1 syntax, including **Record (struct)**, **Optional types**, **async/await**, **error handling**, **module import**, and more.

---

## Table of Contents

1. [Installation & Running](#installation--running)
2. [Core Structure](#core-structure)
3. [Data Types](#data-types)
4. [Optional Types & Null Value](#optional-types--null-value)
5. [Variable Declaration](#variable-declaration)
6. [Expressions](#expressions)
7. [Statements](#statements)
8. [Control Flow](#control-flow)
9. [Functions](#functions)
10. [Asynchronous Programming](#asynchronous-programming)
11. [Arrays](#arrays)
12. [Record (struct)](#record-struct)
13. [Type Casting](#type-casting)
14. [Comments](#comments)
15. [Error Handling](#error-handling)
16. [Module Import](#module-import)
17. [Full Example](#full-example)

---

## Installation & Running

1. Clone and build:

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

## Core Structure

A PawScript program consists of statements and function declarations executed in order.

---

## Data Types

* **Primitive types**: `Int`, `Long`, `Float`, `Double`, `Bool`, `Char`, `String`
* **Generics**: `Array<T>`
* **Special types**: `Any` (dynamic), `Optional<T>` (nullable, can also be written `T?`)

---

## Optional Types & Null Value

PawScript supports optional types to represent possibly missing values.

* Declare an optional type by appending `?`, e.g. `Int?` is `Optional<Int>`.
* The null literal is `nopaw`.
* Assigning `nopaw` to a non-optional type is a compile error.
* Example:

  ```paw
  let maybeNum: Int? = nopaw
  if maybeNum == nopaw {
    say "No number provided"
  }
  ```

---

## Variable Declaration

```paw
let x: Int = 10
let y: Int? = nopaw    # optional type
x = x + 1               # reassignment
```

---

## Expressions

* Arithmetic: `+ - * / %`
* Comparison: `== != < <= > >=`
* Logic: `&& || !`
* String concat: `"Hi " + name + "!"`
* Await: `await <asyncCall>`
* Grouping: `(a + b) * c`

---

## Statements

* Declaration/assignment: `let` / `=`
* Output: `say <expr>`
* Input: `ask "prompt"` or `let x: String <- ask "?"`
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

* `break` exits the nearest loop.
* `continue` skips to the next iteration.

---

## Functions

```paw
fun name(a: Int, b: Float): String {
  return "…"
}
let s: String = name(1, 2.5)
```

---

## Asynchronous Programming

PawScript supports defining asynchronous functions and awaiting their results, enabling non‑blocking I/O and concurrent tasks.

### Async Functions

To define an asynchronous function, prefix the signature with the `async` keyword:

```paw
async fun fetchData(url: String): String {
  // ... perform asynchronous operations ...
  bark "Fetched data from " + url
  return "data"
}
```

* `async` must appear before the `fun` keyword, not after the return type.
* Async functions return a `Future<T>` internally.
* You can store or pass async functions as values.

### Awaiting

Use `await` to suspend execution until a `Future` completes:

```paw
let content: String = await fetchData("http://example.com/data")
say "Received: " + content
```

* `await` can only be used within an async function or at top‑level script.
* If applied to a non‑`Future` value, `await` returns it immediately.
* Attempting to `await` outside an async context will be a compile‑time error.

### Calling Async Functions Without Await

Calling an async function without `await` returns the `Future` object itself:

```paw
let fut = fetchData("http://example.com")
say "Got future: " + fut   # prints a Future placeholder
```

* Store or pass the future for later awaiting.
* Futures can be composed or passed to other async calls.

---

## Arrays

```paw
let a: Array<Int> = [1,2,3]
say a[0]        # index access
say a.length    # length property
```

---

## Record (struct)

PawScript now supports **Record** (struct) for user-defined composite data.

### Definition

Use the `record` keyword:

```paw
record Point {
  x: Int
  y: Int
}
```

* `Point` has two fields: `x: Int` and `y: Int`.
* Field names and types are verified at compile time.

### Initialization

Provide values by field name:

```paw
let p: Point = Point { y: 4, x: 3 }
```

* Order may differ but all fields must be provided.

### Field Access

Use dot syntax:

```paw
say p.x       # 3
say p.y       # 4
```

---

## Type Casting

Explicit casts with `as`:

```paw
let i: Int = 3
let f: Float = i as Float
say f + 1.5
```

* Supports `Int ↔ Long ↔ Float ↔ Double`.
* Invalid casts are compile errors.

---

## Comments

```paw
# single-line comment
let x: Int = 5   # end-of-line comment
```

---

## Error Handling

| Keyword  | Purpose       |
| -------- | ------------- |
| `bark`   | throw error   |
| `sniff`  | try block     |
| `snatch` | catch block   |
| `lastly` | finally block |

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

```paw
import utils.math       # binds module to `math` (last path segment)
import utils.math as m  # binds module to alias `m`
```

* Access functions and constants via the module name or alias.

---

## Full Example

```paw
import utils.math as m
import string

say "=== Record & Async Tests ==="

// Record demo
record Point { x: Int, y: Int }
let p: Point = Point { y: 4, x: 3 }
say "p.x + p.y = " + (p.x + p.y)

// Async demo
fun fetchData(url: String): String async {
  bark "network not implemented"
}
let result: String = await fetchData("http://example.com")

// Loop with break/continue
let sum: Int = 0
loop i in 1..10 {
  if i == 5 {
    continue
  }
  if i == 8 {
    break
  }
  sum = sum + i
}
say "sum = " + sum
```

Happy coding in PawScript — cute and powerful!
