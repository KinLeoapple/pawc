# 🐾 PawScript 入门指南

欢迎使用 **PawScript** —— 一门「可爱又实用」的静态类型函数式脚本语言。本 README 涵盖 v0.1 的全部语法，并补充了三大新特性：**类型转换**、**异常处理** 和 **模块导入**。同时新增对**可空类型（Optional）**与**空值（null / nopaw）**的说明。

---

## 目录

1. [安装与运行](#安装与运行)
2. [基础结构](#基础结构)
3. [数据类型](#数据类型)
4. [可空类型与空值](#可空类型与空值)
5. [变量声明](#变量声明)
6. [表达式](#表达式)
7. [语句](#语句)
8. [流程控制](#流程控制)
9. [函数](#函数)
10. [数组](#数组)
11. [类型转换](#类型转换)
12. [注释](#注释)
13. [异常处理](#异常处理)
14. [模块导入](#模块导入)
15. [完整示例](#完整示例)

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

## 基础结构

PawScript 程序由语句和函数声明组成，按顺序执行。

---

## 数据类型

- **基本类型**
    - 整数：`Int`、`Long`
    - 浮点：`Float`、`Double`
    - 其他：`Bool`、`Char`、`String`
- **泛型**：`Array<T>`
- **特殊**：
    - `Any`（动态类型）
    - `Optional<T>` 可空类型（也可写作 `T?`）

---

## 可空类型与空值

PawScript 支持可空（Optional）类型，用于表示可能没有值的情况。

- 声明可空类型：在类型后加 `?`，例如 `Int?` 等价于 `Optional<Int>`。
- 空值字面量：使用关键字 `nopaw`，对应运行时的 `null`。
- 可空类型变量可以被赋予空值：
  ```paw
  let maybeNum: Int? = nopaw
  if maybeNum == nopaw {
    say "No number provided"
  }
  ```
- 对可空类型进行解包需要先检查是否为空；将 `nopaw` 赋给非可空类型会在静态检查时报错。

---

## 变量声明

```paw
let x: Int = 10
let y: Int? = nopaw    # 可空类型
x = x + 1             # 重赋值
```

---

## 表达式

- 算术：`+ - * / %`
- 比较：`== != < <= > >=`
- 逻辑：`&& || !`
- 字符串拼接：`"Hi " + name + "!"`
- 分组：`(a + b) * c`
- 可空比较：可与 `nopaw` 比较

---

## 语句

- 声明 / 赋值：`let` / `=`
- 输出：`say <expr>`
- 输入：`ask <"prompt">` 或 `let x: String <- ask "?"`
- 返回：`return <expr>` 或 `return`

---

## 流程控制

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

## 函数

```paw
fun name(a: Int, b: Float): String {
  return "…"
}
let s: String = name(1, 2.5)
```

---

## 数组

```paw
let a: Array<Int> = [1,2,3]
say a[0]        # 下标访问
say a.length    # 属性
```

---

## 类型转换

使用 `as`：

```paw
let i: Int = 3
let f: Float = i as Float   # Int → Float
say f + 1.5
```

- 支持 Int/Long/Float/Double 之间
- 相同类型转换为无操作
- 非法转换编译报错

---

## 注释

```paw
# 单行注释
let x: Int = 5   # 行尾注释
```

---

## 异常处理

| 关键字    | 功能        |
|----------|------------|
| `bark`   | 抛出异常    |
| `sniff`  | try 块      |
| `snatch` | catch 块    |
| `lastly` | finally 块  |

### 抛出

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

## 模块导入

按路径导入 `.paw` 文件，可选别名。

```paw
import utils.math       # 默认别名 “math”
import utils.math as m  # 别名 “m”
```

- 访问成员：`m.square(5)` 或 `utils.math.PI`

---

## 完整示例

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

祝你编程愉快！
