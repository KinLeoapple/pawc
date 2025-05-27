# PawScript

PawScript 是一门既萌又实用的高性能强类型脚本语言，旨在实现简洁、富有表现力和安全性。它具有静态类型检查、现代异步支持和丰富的标准库。PawScript 支持同步和异步编程范式、先进的控制流机制以及强大的模块系统。

---

## 目录

1. [快速开始](#快速开始)
2. [语言概览](#语言概览)

   * [基本类型 & 字面量](#基本类型--字面量)
   * [变量 & 赋值](#变量--赋值)
   * [表达式](#表达式)
   * [语句](#语句)
   * [控制流](#控制流)
   * [错误处理](#错误处理)
   * [函数 & 异步](#函数--异步)
   * [协议 & 记录](#协议--记录)
   * [模块 & 导入](#模块--导入)

---

## 快速开始

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

## 语言概览

### 基本类型 & 字面量

* **原始类型**：`Int`、`Long`、`Float`、`Double`、`Bool`、`Char`、`String`
* **可选类型**：`T?` 或 `Optional<T>`；空值为 `nopaw`
* **数组**：`Array<T>`，例如 `Array<Int>`
* **字面量详情**：

   * **字符串 & 字符**：支持 `\n`、`\r`、`\t`、`\\`、`\"`、Unicode 转义 `\u{XXXX}`
   * **数字后缀**：`123L`（Long）、`3.14F`（Float）、`2.718D`（Double）

### 变量 & 赋值

```paw
let x: Int = 10
let y: Int? = nopaw
x = x + 1
let name: String <- ask "Your name?"
```

* 使用 `let <id>: <Type> = <expr>` 声明变量
* 使用 `<id> = <expr>` 重新赋值
* 使用 `<-` 将 `ask` 的输入绑定到变量

### 表达式

* 算术运算：`+`、`-`、`*`、`/`、`%`
* 比较运算：`==`、`!=`、`<`、`<=`、`>`、`>=`
* 逻辑运算：`&&`、`||`、`!`
* 字符串拼接：`"Hi " + name`
* 字符串插值：`"Hello, ${name.toUpperCase()}!"`
* 分组运算：`(a + b) * c`
* 类型转换：`expr as TargetType`
* 异步等待：`await asyncCall()`

### 语句

* **输出**：`say <expr>`
* **输入**：`ask "prompt"` 及其绑定形式
* **返回**：`return <expr>` 或 `return`
* **抛错**：`bark <expr>`
* **表达式语句**：调用函数或使用 `await`
* **注释**：`# this is a comment`

### 控制流

```paw
if cond {
  …
} else if cond2 {
  …
} else {
  …
}

loop { … }             // 无限循环
loop cond { … }        // 条件循环
loop i in 1..10 { … }  // 范围循环
loop item in array { }  // 遍历循环
break
continue
```

### 错误处理

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

### 函数 & 异步

* **同步函数**：

  ```paw
  fun add(a: Int, b: Int): Int {
    return a + b
  }
  ```
* **异步函数**：

  ```paw
  async fun fetch(): Int {
    return await http.get()
  }
  ```
* 可省略返回类型，默认为 `Void`

### 协议 & 记录

```paw
tail Greeter {
  fun greet(self, name: String): String
  async fun fetch(): Int
}

record Person: Greeter {
  name: String
  age: Int
  fun greet(self, name: String): String {
    return "Hello, ${self.name}!"
  }
}
```

### 模块 & 导入

```paw
import utils.math
import utils.math as m
```

---
