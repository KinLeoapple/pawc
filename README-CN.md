Thought for a couple of seconds


Thought for 5 seconds


# 🐾 PawScript 入门指南

欢迎使用 **PawScript** — 一门“萌 yet 实用”的静态类型函数式脚本语言。本指南涵盖 v0.1 的全部语法，包括**记录（struct）**、**可选类型**、**异步/await**、**错误处理**、**模块导入**等内容。

---

## 目录

1. [安装与运行](#安装与运行)
2. [CLI 栈大小选项](#cli-栈大小选项)
3. [核心结构](#核心结构)
4. [数据类型](#数据类型)
5. [可选类型 & 空值](#可选类型--空值)
6. [变量声明](#变量声明)
7. [表达式](#表达式)
8. [语句](#语句)
9. [控制流](#控制流)
10. [函数](#函数)
11. [异步编程](#异步编程)
12. [数组](#数组)
13. [记录（struct）](#记录struct)
14. [类型转换](#类型转换)
15. [注释](#注释)
16. [错误处理](#错误处理)
17. [模块导入](#模块导入)
18. [完整示例](#完整示例)

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

## CLI 栈大小选项

PawScript 解释器支持通过命令行参数调整主线程的备份栈大小（单位 MiB），以应对深度递归场景。

```bash
# 默认：主线程备份栈 1 MiB
target/release/pawc script.paw

# 自定义：主线程备份栈 4 MiB
target/release/pawc --stack-size 4 script.paw
```

---

## 核心结构

PawScript 程序由语句和函数声明按顺序执行组成。

---

## 数据类型

* **原始类型**：`Int`, `Long`, `Float`, `Double`, `Bool`, `Char`, `String`
* **泛型**：`Array<T>`
* **特殊类型**：`Any`（动态类型），`Optional<T>`（可空类型，可写作 `T?`）

---

## 可选类型 & 空值

PawScript 支持可选类型来表示可能缺失的值。

* 在类型后追加 `?` 来声明可选类型，例如 `Int?` 等同于 `Optional<Int>`。
* 空字面量为 `nopaw`。
* 将 `nopaw` 赋值给非可选类型会导致编译期错误。

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
let x: Int = 10
let y: Int? = nopaw    # 可选类型
x = x + 1               # 重新赋值
```

---

## 表达式

* 算术：`+ - * / %`
* 比较：`== != < <= > >=`
* 逻辑：`&& || !`
* 字符串拼接：`"Hi " + name + "!"`
* Await：`await <asyncCall>`
* 分组：`(a + b) * c`

---

## 语句

* 声明/赋值：`let` / `=`
* 输出：`say <expr>`
* 输入：`ask "prompt"` 或 `let x: String <- ask "?"`
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
loop item in array { … }
```

* `break` 退出最近的循环。
* `continue` 跳到下一次迭代。

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

PawScript 支持定义异步函数并使用 `await`，实现非阻塞 I/O 和并发任务。

### 异步函数

在函数前添加 `async` 关键字：

```paw
async fun fetchData(url: String): String {
  # 模拟异步操作
  bark "network not implemented"
  return "data"
}
```

* `async` 必须出现在 `fun` 之前。
* 异步函数内部返回 `Future<T>`。

### Await

使用 `await` 等待 `Future` 完成：

```paw
let content: String = await fetchData("http://example.com/data")
say "Received: " + content
```

* `await` 可在顶层或异步函数中使用。
* 对非 Future 应用 `await` 会原样返回该值。

---

## 数组

```paw
let a: Array<Int> = [1, 2, 3]
say a[0]        # 索引访问
say a.length()    # 长度属性
```

---

## 记录（struct）

PawScript 支持用户自定义复合类型 **Record**（struct）。

### 定义

```paw
record Point {
  x: Int
  y: Int
}
```

* 初始化时必须提供所有字段。

### 初始化

```paw
let p: Point = Point { y: 4, x: 3 }
```

* 字段顺序可任意。

### 访问

```paw
say p.x    # 3
say p.y    # 4
```

---

## 类型转换

使用 `as` 进行显式转换：

```paw
let i: Int = 3
let f: Float = i as Float
say f + 1.5
```

* 支持 `Int ↔ Long ↔ Float ↔ Double`。
* 无效转换为编译期错误。

---

## 注释

```paw
# 单行注释
let x: Int = 5   # 行尾注释
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
  say "Caught: " + e
} lastly {
  say "Cleanup"
}
```

---

## 模块导入

```paw
import utils.math       # 绑定模块到 `math`
import utils.math as m  # 绑定模块到别名 `m`
```

* 通过模块名或别名访问其中的函数/常量。

---

## 完整示例

```paw
import utils.math as m
import string

say "=== Record & Async Example ==="

# 记录
record Point { x: Int, y: Int }
let p: Point = Point { y: 4, x: 3 }
say "p.x + p.y = " + (p.x + p.y)

# 异步
async fun fetchData(url: String): String {
  bark "network not implemented"
  return "data"
}
let result: String = await fetchData("http://example.com")

# 带 break/continue 的循环
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

祝您在 PawScript 的世界里编程愉快！
