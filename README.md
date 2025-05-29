# PawScript

PawScript is a cute yet practical, high-performance, strongly-typed scripting language designed for simplicity, expressiveness, and safety. It features statically checked types, modern async support, and a rich standard library. PawScript supports both synchronous and asynchronous programming paradigms, advanced control flow, and a robust module system.

---

## Table of Contents

1. [Quick Start](#quick-start)
2. [Language Overview](#language-overview)

    * [Basic Types & Literals](#basic-types--literals)
    * [Variables & Assignment](#variables--assignment)
    * [Expressions](#expressions)
    * [Statements](#statements)
    * [Control Flow](#control-flow)
    * [Error Handling](#error-handling)
    * [Functions & Async](#functions--async)
    * [Protocols & Records](#protocols--records)
    * [Modules & Imports](#modules--imports)

---

## Quick Start

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

## Language Overview

### Basic Types & Literals

* **Primitive Types**: `Int`, `Long`, `Float`, `Double`, `Bool`, `Char`, `String`
* **Optional Types**: `T?`; empty value is `nopaw`
* **Arrays**: `Array<T>`, e.g. `Array<Int>`
* **Literal Details**:

    * **Strings & Chars**: support `\n`, `\r`, `\t`, `\\`, `\"`, Unicode escapes `\u{XXXX}`
    * **Numeric Suffixes**: `123L` (Long), `3.14F` (Float), `2.718D` (Double)

### Variables & Assignment

```paw
let x: Int = 10
let y: Int? = nopaw
x = x + 1
let name: String = ask "Your name?"
```

* `let <id>: <Type> = <expr>` to declare
* `<id> = <expr>` to reassign
* `ask` binds input when using `<-`

### Expressions

* Arithmetic: `+`, `-`, `*`, `/`, `%`
* Comparison: `==`, `!=`, `<`, `<=`, `>`, `>=`
* Logical: `&&`, `||`, `!`
* String concat: `"Hi " + name`
* String interpolation: `"Hello, ${name.toUpperCase()}!"`
* Grouping: `(a + b) * c`
* Type cast: `expr as TargetType`
* Async await: `await asyncCall()`

### Statements

* **Output**: `say <expr>`
* **Input**: `ask "prompt"` and binding form
* **Return**: `return <expr>` or `return`
* **Throw**: `bark <expr>`
* **Expression statements**: call functions or await
* **Comments**: `# this is a comment`

### Control Flow

```paw
if cond {
  …
} else if cond2 {
  …
} else {
  …
}

loop { … }             // infinite
loop cond { … }        // while
loop i in 1..10 { … }  // range
loop item in array { }  // iterate
break
continue
```

### Error Handling

```paw
sniff {
  …
} snatch (e) {
  …
} lastly {
  …
}
bark "error message"
```

### Functions & Async

* **Sync**:

  ```paw
  fun add(a: Int, b: Int): Int {
    return a + b
  }
  ```
* **Async**:

  ```paw
  async fun fetch(): Int {
    return await http.get()
  }
  ```
* Omit return type for `Void`

### Protocols & Records

```paw
tail Greeter {
  fun greet(name: String): String
  async fun fetch(): Int
}

record Person: Greeter {
  name: String,
  age: Int
  fun greet(name: String): String {
    return "Hello, ${self.name}!"
  }
}

let p: Person = Person { name: "Paw", age: 18 }
say p.greet("visitor")
```

### Modules & Imports

```paw
import utils.math
import utils.math as m
```

---
