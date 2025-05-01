# 🐾 PawScript Quickstart Guide

Welcome to **PawScript**—a cute yet practical, statically-typed scripting language with functional flavor. This README covers all syntax and features in v0.1.

---

## Table of Contents

1. [Getting Started](#getting-started)  
2. [Program Structure](#program-structure)  
3. [Data Types](#data-types)  
4. [Variables & Input](#variables--input)  
5. [Expressions](#expressions)  
6. [Statements](#statements)  
7. [Control Flow](#control-flow)  
8. [Functions](#functions)  
9. [Arrays](#arrays)  
10. [Comments](#comments)  
11. [Complete Example](#complete-example)

---

## Getting Started

1. **Clone and enter**:
   ```bash
   git clone https://github.com/your-org/pawscript.git
   cd pawscript
   ```
2. **Run in interpreter mode**:
   ```bash
   cargo run -- path/to/script.paw
   ```
3. **Build the compiler (`pawc`)**:
   ```bash
   cargo build --release
   ./target/release/pawc path/to/script.paw
   ```

---

## Program Structure

A PawScript program is a sequence of top-level statements: declarations, function definitions, loops, etc. Execution is sequential: parse → static type check → interpret or emit C/C++.

---

## Data Types

| Type         | Description                       |
| ------------ | --------------------------------- |
| `Int`        | 32-bit signed integer             |
| `Long`       | 64-bit signed integer             |
| `Float`      | 32-bit floating-point              |
| `Double`     | 64-bit floating-point              |
| `Bool`       | Boolean (`true` / `false`)        |
| `Char`       | Single character                  |
| `String`     | UTF-8 string                      |
| `Array<T>`   | Dynamic array of elements of `T`  |
| `Any`        | Untyped or unknown                |

---

## Variables & Input

- **Immutable**: `let`
- **Mutable**: `var`

```paw
let x: Int    = 42
var y: Float  = 3.14

# Prompt the user, bind result to `name`
let name: String <- ask "What's your name?"
```

- `let … = …` for normal assignment
- `let … <- ask "…"` to prompt and capture input

---

## Expressions

```paw
# Arithmetic
1 + 2 * (3 - 4) / 5 % 2

# Comparison
a == b
a != b
a <  b
a >= b

# Logical
flag && (count > 0) || !done

# Function call
sum(1, 2, 3)

# Array indexing
[1, 2, 3][0]
arr[i % len(arr)]

# String concatenation (auto to_string)
"Hello, " + name + "!"

# Unary
-x
!flag
```

---

## Statements

### Output & Prompt

```paw
say "The value is " + x
ask "Press Enter to continue"
let answer: Int <- ask "Enter a number:"
```

### Assignment

```paw
let n: Int = 0
n = addOne(n)
```

### Return

```paw
return          # inside a function
return expr     # return a value
```

---

## Control Flow

### Conditional

```paw
if a == 0 {
    say "zero"
} else if a < 0 {
    say "negative"
} else {
    say "positive"
}
```

### Loops

```paw
# Infinite loop
loop forever {
    say "looping…"
    break
}

# While-style loop
loop x > 0 {
    x = x - 1
}

# Range loop
loop i in 0..5 {
    say i
}

# break and continue
if i == 3 { continue }
if i == 4 { break }
```

---

## Functions

```paw
# With return
fun add(x: Int, y: Int): Int {
    return x + y
}

# Void function
fun greet(name: String) {
    say "Hello, " + name + "!"
}

# Call
let r: Int = add(5, 7)
greet("Paw")
```

- Parameters and return type **must** be annotated.
- Supports recursion and nested calls.

---

## Arrays

```paw
# Declare an array
let nums: Array<Int> = [10, 20, 30]

# Access elements
say "first=" + nums[0]

# Mutable array
var arr: Array<String> = ["a", "b", "c"]
arr[1] = "B"
```

Type is `Array<ElementType>`. Literals use `[ ... ]`. Zero-based indexing.

---

## Comments

- Single-line comments start with `#`

```paw
# This is a comment
let x: Int = 100  # Inline comment
```

---

## Complete Example

```paw
# Fibonacci generator

ask "Press Enter to start…"

fun fib(n: Int): Int {
    if n < 2 {
        return n
    } else {
        return fib(n-1) + fib(n-2)
    }
}

let count: Int <- ask "How many terms? "
say "Fibonacci sequence:"

loop i in 0..count {
    say fib(i)
}
```

Save as `fib.paw` and run:

```bash
pawc fib.paw
```

Enjoy PawScript!  
Expand with `pawfmt`, editor plugins, or custom bootstrapping and extensions.