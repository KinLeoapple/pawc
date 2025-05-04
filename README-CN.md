# 🐾 PawScript 入门指南

欢迎使用 **PawScript** —— 一种“可爱又实用”的静态类型函数式脚本语言。本指南覆盖 v0.1 版所有语法，包括 **记录（Record/结构体）**、**可选类型**、**异步/等待**、**错误处理**、**模块导入** 等。

---

## 目录

1. [安装与运行](#安装与运行)
2. [核心结构](#核心结构)
3. [数据类型](#数据类型)
4. [可选类型与 Null](#可选类型与-null)
5. [变量声明](#变量声明)
6. [表达式](#表达式)
7. [语句](#语句)
8. [控制流](#控制流)
9. [函数](#函数)
10. [异步编程](#异步编程)
11. [数组](#数组)
12. [记录（Record/结构体）](#记录record结构体)
13. [类型转换](#类型转换)
14. [注释](#注释)
15. [错误处理](#错误处理)
16. [模块导入](#模块导入)
17. [完整示例](#完整示例)

---

## 安装与运行

1. 克隆并编译：

   ```bash
   git clone https://github.com/KinLeoapple/pawc.git
   cd pawc
   cargo build --release
   ```
2. 运行脚本：

   ```bash
   target/release/pawc hello.paw
   ```

---

## 核心结构

PawScript 程序由按顺序执行的语句和函数声明组成。

---

## 数据类型

* **基本类型**：`Int`、`Long`、`Float`、`Double`、`Bool`、`Char`、`String`
* **泛型类型**：`Array<T>`
* **特殊类型**：`Any`（动态类型）、`Optional<T>`（可空，可写作 `T?`）

---

## 可选类型与 Null

PawScript 支持可选类型来表示可能不存在的值。

* 可选类型通过在类型后加 `?` 声明，例如 `Int?` 等价于 `Optional<Int>`。
* 空值字面量为 `nopaw`。
* 将 `nopaw` 赋给非可选类型会导致编译错误。

示例：

```paw
let maybeNum: Int? = nopaw
if maybeNum == nopaw {
  say "未提供数字"
}
```

---

## 变量声明

```paw
let x: Int = 10       # 声明并初始化
let y: Int? = nopaw   # 可选类型
x = x + 1              # 重新赋值
```

---

## 表达式

* 算术运算：`+ - * / %`
* 比较运算：`== != < <= > >=`
* 逻辑运算：`&& || !`
* 字符串拼接：`"Hi " + name + "!"`
* 等待结果：`await <asyncCall>`
* 分组：`(a + b) * c`

---

## 语句

* 声明/赋值：`let` / `=`
* 输出：`say <expr>`
* 输入：`ask "提示"` 或 `let x: String <- ask "?"`
* 返回：`return <expr>` 或 `return`

---

## 控制流

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

* `break` 跳出最近的循环。
* `continue` 跳过本次迭代。

---

## 函数

```paw
fun add(a: Int, b: Int): Int {
  return a + b
}
let result: Int = add(1, 2)
```

---

## 异步编程

PawScript 支持定义异步函数并等待其结果，实现非阻塞 I/O 和并发任务。

### 异步函数

在 `fun` 前添加 `async` 关键字：

```paw
async fun fetchData(url: String): String {
  // ... 异步操作 ...
  bark "Fetched data from " + url
  return "data"
}
```

* `async` 必须写在 `fun` 前。
* 异步函数内部返回类型视作 `Future<T>`。
* 可以将异步函数作为值传递或存储。

### 等待结果

使用 `await` 暂停执行直到 `Future` 完成：

```paw
let content: String = await fetchData("http://example.com/data")
say "收到: " + content
```

* `await` 可在脚本顶层或异步函数内使用。
* 对非 `Future` 值应用 `await` 时，立即返回该值。
* 在非异步上下文使用 `await` 会报编译错误。

### 不等待的调用

不使用 `await` 调用异步函数会返回 `Future` 对象：

```paw
let fut = fetchData("http://example.com")
say "Future: " + fut
```

* 可将 `Future` 存储或传递以待后续 `await`。

---

## 数组

```paw
let a: Array<Int> = [1, 2, 3]
say a[0]      # 索引访问
say a.length  # 长度属性
```

---

## 记录（Record/结构体）

PawScript 支持 **Record**（类似结构体）定义复合数据。

### 定义

使用 `record` 关键字：

```paw
record Point {
  x: Int
  y: Int
}
```

* `Point` 类型包含字段 `x` 和 `y`。

### 初始化

字段名赋值：

```paw
let p: Point = Point { y: 4, x: 3 }
```

* 字段顺序可与定义顺序不同，但必须提供所有字段。

### 字段访问

点式语法：

```paw
say p.x  # 3
say p.y  # 4
```

---

## 类型转换

使用 `as` 关键字：

```paw
let i: Int = 3
let f: Float = i as Float
say f + 1.5
```

* 支持 `Int ↔ Long ↔ Float ↔ Double` 之间转换。
* 无效转换编译时报错。

---

## 注释

```paw
# 单行注释
let x: Int = 5  # 行尾注释
```

---

## 错误处理

| 关键字      | 用途        |
| -------- | --------- |
| `bark`   | 抛出错误      |
| `sniff`  | try 块     |
| `snatch` | catch 块   |
| `lastly` | finally 块 |

```paw
sniff {
  …
} snatch (e) {
  say "捕获错误: " + e
} lastly {
  say "清理完成"
}
```

---

## 模块导入

```paw
import utils.math       # 绑定为 math
import utils.math as m  # 绑定为 m
```

* 不写别名时，模块名取最后一段路径。
* 使用 `模块名.成员` 访问函数和常量。

---

## 完整示例

```paw
import utils.math as m
import string

say "=== Record & Async 示例 ==="

// Record 示例\ nrecord Point { x: Int, y: Int }
let p: Point = Point { y: 4, x: 3 }
say "p.x + p.y = " + (p.x + p.y)

// Async 示例
async fun fetchData(url: String): String {
  bark "network not implemented"
}
let result: String = await fetchData("http://example.com")

// 循环示例
let sum: Int = 0
loop i in 1..10 {
  if i == 5 { continue }
  if i == 8 { break }
  sum = sum + i
}
say "sum = " + sum
```

祝您在 PawScript 中编程愉快！ —— 可爱而强大！
