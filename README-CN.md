# 🐾 PawScript 新手指南

欢迎使用 **PawScript** —— 一门「可爱又实用」、支持静态类型、函数式语法风格的脚本语言。本 README 覆盖截至 v0.1 的所有语法要素和风格规范。

---

## 目录

1. [安装与运行](#安装与运行)  
2. [基本结构](#基本结构)  
3. [数据类型](#数据类型)  
4. [变量声明](#变量声明)  
5. [表达式](#表达式)  
6. [语句](#语句)  
7. [控制流](#控制流)  
8. [函数](#函数)  
9. [数组](#数组)  
10. [注释](#注释)  
11. [完整示例](#完整示例)

---

## 安装与运行

1. 克隆仓库并进入目录：  
   ```bash
   git clone https://github.com/KinLeoapple/pawc.git
   cd pawc
   ```
2. 编译并运行（解释器模式）：
   ```bash
   cargo run -- path/to/script.paw
   ```
3. 或者构建可执行 `pawc`：
   ```bash
   cargo build --release
   ./target/release/pawc path/to/script.paw
   ```

---

## 基本结构

一个 PawScript 程序本质上是一系列顶层语句（`Statement`），可以是变量/常量声明、函数定义、流程控制等。  
执行流程自上而下，支持静态类型检查和运行时解释／生成 C/C++ 代码。

---

## 数据类型

| 名称    | 描述                                |
|-------|-----------------------------------|
| `Int`   | 32 位有符号整数                         |
| `Long`  | 64 位有符号整数                         |
| `Float` | 32 位浮点数                            |
| `Double`| 64 位浮点数                            |
| `Bool`  | 布尔值 (`true`/`false`)              |
| `Char`  | 单字符字面量                          |
| `String`| 可变长 UTF-8 字符串                   |
| `Array<T>` | 元素类型为 `T` 的动态数组            |
| `Any`   | 任意类型（类型检查时跳过）              |

---

## 变量声明

- **不可变**：`let`

```paw
let x: Int    = 42

# 声明式输入
let name: String <- ask "What's your name?"
```

- `let … = …` 普通赋值
- `let … <- ask "…"` 声明式输入，返回值绑定到变量

---

## 表达式

```paw
# 算术
1 + 2 * (3 - 4) / 5 % 2

# 比较
a == b
a != b
a <  b
a >= b

# 逻辑
flag && (count > 0) || !done

# 函数调用
sum(1, 2, 3)

# 数组
[1, 2, 3][0]      # 取第 0 个元素
arr[ i % len(arr) ]

# 字符串拼接（自动 to_string）
"Hello, " + name + "!"

# 一元
-x
!flag
```

---

## 语句

### 输出与输入

```paw
say "Value is " + x      # 输出到终端
ask "Press Enter to continue"  # 直接弹提示
let answer: Int <- ask "Enter a number:"
```

### 赋值

```paw
let n: Int = 0
n = addOne(n)
```

### 返回

```paw
return          # 仅在函数内部有效
return expr     # 返回值
```

---

## 控制流

### 条件分支

```paw
if a == 0 {
    say "zero"
} else if a < 0 {
    say "negative"
} else {
    say "positive"
}
```

### 循环

```paw
# 无限循环
loop forever {
    say "looping…"
    break
}

# 条件循环（相当于 while）
loop x > 0 {
    x = x - 1
}

# 范围循环
loop i in 0..5 {
    say i
}

# 跳出与继续
if i == 3 { continue }
if i == 4 { break }
```

---

## 函数

```paw
# 带返回值
fun add(x: Int, y: Int): Int {
    return x + y
}

# 无返回值（Void 可省略）
fun greet(name: String) {
    say "Hello, " + name + "!"
}

# 调用
let r: Int = add(5, 7)
greet("Paw")
```

- 参数与返回类型**必须**注解
- 支持嵌套调用与递归

---

## 数组

```paw
# 定义数组
let nums: Array<Int> = [10, 20, 30]

# 访问
say "first=" + nums[0]
```

> **注意**：数组类型标注为 `Array<元素类型>`，字面量用 `[...]`，索引从 `0` 开始。

---

## 注释

- **单行**：`#` 开头

```paw
# 这是单行注释
let foo: Int = 100  # 行尾也可注释
```

---

## 完整示例

```paw
# 计算并展示斐波那契数列

ask "Press Enter to start…"

fun fib(n: Int): Int {
    if n < 2 {
        return n
    } else {
        return fib(n-1) + fib(n-2)
    }
}

let count: Int <- ask "How many terms? "
say "Fibonacci:"

loop i in 0..count {
    say fib(i)
}
```

---

祝你用得开心！  
如需扩展：
- 格式化工具 `pawfmt`
- VSCode 语法高亮/代码片段
- 支持自举 (bootstrapping) & 插件拓展机制