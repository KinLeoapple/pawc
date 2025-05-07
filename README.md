# 🐾 PawScript Getting Started Guide

Welcome to **PawScript** — a “cute yet practical” statically‑typed, functional scripting language. This README covers all v0.1 syntax, including **Record (struct)**, **Optional types**, **async/await**, **error handling**, **module import**, and more.

---

## Table of Contents

1. [Installation & Running](#installation--running)
2. [CLI Stack‑Size Options](#cli-stack‑size-options)
3. [Core Structure](#core-structure)
4. [Data Types](#data-types)
5. [Optional Types & Null Value](#optional-types--null-value)
6. [Variable Declaration](#variable-declaration)
7. [Expressions](#expressions)
8. [Statements](#statements)
9. [Control Flow](#control-flow)
10. [Functions](#functions)
11. [Asynchronous Programming](#asynchronous-programming)
12. [Arrays](#arrays)
13. [Record (struct)](#record-struct)
14. [Type Casting](#type-casting)
15. [Comments](#comments)
16. [Error Handling](#error-handling)
17. [Module Import](#module-import)
18. [Full Example](#full-example)

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

## CLI Stack‑Size Options

PawScript Interpreter supports adjusting the backup stack size for the main thread via a CLI flag (in MiB) to accommodate deep‑recursion scenarios.

```bash
# Default: main‑thread backup stack 1 MiB
target/release/pawc script.paw

# Custom: main‑thread backup stack 4 MiB
target/release/pawc --stack-size 4 script.paw
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
* Assigning `nopaw` to a non‑optional type is a compile‑time error.
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
* String concatenation: `"Hi " + name + "!"`
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
loop item in array { … }
```

* `break` exits the nearest loop.
* `continue` skips to the next iteration.

---

## Functions

```paw
fun add(a: Int, b: Int): Int {
  return a + b
}
let result: Int = add(1, 2)
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

* `async` must appear before the `fun` keyword.
* Async functions return a `Future<T>` internally.
* You can store or pass async functions as values.

### Await

Use `await` to suspend until a `Future` completes:

```paw
let content: String = await fetchData("http://example.com/data")
say "Received: " + content
```

* `await` may be used at top‑level or within async functions.
* Awaiting a non‑Future returns the value unchanged.

---

## Arrays

```paw
let a: Array<Int> = [1, 2, 3]
say a[0]        # index access
say a.length    # length property
```

---

## Record (struct)

PawScript supports user‑defined composite types called **Record** (struct).

### Definition

```paw
record Point {
  x: Int
  y: Int
}
```

* Fields must all be provided at initialization.

### Initialization

```paw
let p: Point = Point { y: 4, x: 3 }
```

* Field order is arbitrary.

### Access

```paw
say p.x    # 3
say p.y    # 4
```

---

## Type Casting

Use `as` for explicit casts:

```paw
let i: Int = 3
let f: Float = i as Float
say f + 1.5
```

* Supports `Int ↔ Long ↔ Float ↔ Double`.
* Invalid casts are compile‑time errors.

---

## Comments

```paw
# single‑line comment
let x: Int = 5   # end‑of‑line comment
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
import utils.math       # binds module to `math`
import utils.math as m  # binds module to alias `m`
```

* Access functions/constants via module name or alias.

---

## Full Example

```paw
import utils.math as m
import string

say "=== Record & Async Example ==="

# Record
record Point { x: Int, y: Int }
let p: Point = Point { y: 4, x: 3 }
say "p.x + p.y = " + (p.x + p.y)

# Async
async fun fetchData(url: String): String {
  bark "network not implemented"
  return "data"
}
let result: String = await fetchData("http://example.com")

# Loop with break/continue
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

Enjoy coding in PawScript — cute yet powerful!
