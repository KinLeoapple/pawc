# 🐾 PawScript 入门指南

欢迎使用 **PawScript** —— 一种“可爱又实用”的静态类型函数式脚本语言。本说明覆盖 v0.1 版的全部语法，包括 **记录（Record/结构体）**、**可选类型**、**异步/等待**、**错误处理**、**模块导入** 等。

---

## 目录

1. [安装与运行](#安装与运行)  
2. [CLI 栈大小参数](#cli-栈大小参数)  
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
````

2. 运行脚本：

```bash
   target/release/pawc hello.paw
   ```

---

## CLI 栈大小参数

PawScript 解释器支持通过命令行参数（MiB 为单位）调整栈大小，以应对深度递归场景。

```bash
# 默认：主线程备用栈 1MiB，Tokio worker 线程栈 1MiB
target/release/pawc script.paw

# 自定义：主线程备用栈 4MiB，worker 线程栈 2MiB
target/release/pawc --stack-size 4 --worker-stack-size 2 script.paw
```

* `--stack-size <MiB>`：当主线程剩余栈 <32KiB 时，扩容到此大小（默认 **1**）。
* `--worker-stack-size <MiB>`：Tokio worker 线程的栈大小（默认 **1**）。

---

## 核心结构

一个 PawScript 程序由按顺序执行的语句和函数声明组成。

---

## 数据类型

* **原始类型**：`Int`, `Long`, `Float`, `Double`, `Bool`, `Char`, `String`
* **泛型**：`Array<T>`
* **特殊类型**：`Any`（动态类型）、`Optional<T>`（可空类型，可写作 `T?`）

---

## 可选类型 & 空值

PawScript 支持可选类型来表示可能缺失的值。

* 在类型后加 `?` 声明可选类型，例如 `Int?` 等同于 `Optional<Int>`。
* 空值字面量为 `nopaw`。
* 将 `nopaw` 赋给非可选类型会报编译错误。
* 示例：

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
* 等待：`await <asyncCall>`
* 分组：`(a + b) * c`

---

## 语句

* 声明/赋值：`let` / `=`
* 输出：`say <expr>`
* 输入：`ask "prompt"` 或 `let x: String ← ask "?"`
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

* `break` 跳出最近的循环。
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

PawScript 支持定义异步函数并等待其结果，实现非阻塞 I/O 和并发任务。

### Async 函数

在函数签名前加 `async` 关键字：

```paw
async fun fetchData(url: String): String {
  bark "网络未实现"
  return "data"
}
```

* `async` 必须写在 `fun` 之前。
* 异步函数内部返回 `Future<T>`。
* 可将异步函数当值存储或传递。

### Await

使用 `await` 等待 `Future` 完成：

```paw
let content: String = await fetchData("http://example.com/data")
say "收到内容：" + content
```

* `await` 只能在异步函数或顶层脚本中使用。
* 对非 `Future` 值使用 `await` 会原样返回该值。

### 不使用 Await 调用 Async

调用异步函数但不 `await`，会得到 `Future` 对象：

```paw
let fut = fetchData("http://example.com")
say "得到 future：" + fut   # 打印 Future 占位
```

* 可将该 Future 存储或稍后再等待。

---

## 数组

```paw
let a: Array<Int> = [1, 2, 3]
say a[0]        # 索引访问
say a.length()    # 长度属性
```

---

## 记录（struct）

PawScript 支持用户定义的复合数据类型 **Record**（结构体）。

### 定义

使用 `record` 关键字：

```paw
record Point {
  x: Int
  y: Int
}
```

* `Point` 包含两个字段：`x: Int` 和 `y: Int`。
* 字段名和类型在编译时检查。

### 初始化

按字段名提供初始值：

```paw
let p: Point = Point { y: 4, x: 3 }
```

* 字段顺序可任意，但必须提供所有字段。

### 访问字段

使用点语法：

```paw
say p.x       # 3
say p.y       # 4
```

---

## 类型转换

使用 `as` 显式转换：

```paw
let i: Int = 3
let f: Float = i as Float
say f + 1.5
```

* 支持 `Int ↔ Long ↔ Float ↔ Double`。
* 无效转换为编译错误。

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
  say "捕获到：" + e
} lastly {
  say "清理工作"
}
```

---

## 模块导入

```paw
import utils.math       # 绑定模块到 `math`
import utils.math as m  # 绑定模块到别名 `m`
```

* 通过模块名或别名访问其函数和常量。

---

## 完整示例

```paw
import utils.math as m
import string

say "=== Record & Async 测试 ==="

// Record 示例
record Point { x: Int, y: Int }
let p: Point = Point { y: 4, x: 3 }
say "p.x + p.y = " + (p.x + p.y)

// Async 示例
async fun fetchData(url: String): String {
  bark "网络未实现"
}
let result: String = await fetchData("http://example.com")

// 带 break/continue 的循环
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

祝您在 PawScript 中编码愉快 — 可爱又强大！
