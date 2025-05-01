# 🐾 PawScript 新手指南

欢迎使用 **PawScript** —— 一门「可爱又实用」、支持静态类型、函数式风格的脚本语言。本 README 覆盖截至 v0.1 的所有语法要素，新增「类型转换」和「异常处理」两大模块。

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
10. [类型转换](#类型转换)
11. [注释](#注释)
12. [异常处理](#异常处理)
13. [完整示例](#完整示例)

---

## 1. 安装与运行

1. 克隆仓库并编译：
   ```bash
   git clone https://github.com/KinLeoapple/pawc.git
   cd pawc
   cargo build --release
   ```
2. 解释执行：
   ```bash
   target/release/pawc run hello.paw
   ```

---

## 2. 基本结构

每个脚本是一系列语句（statement）或函数声明。执行从顶部开始，依次运行。

---

## 3. 数据类型

- 原始类型：`Int`、`Float`、`Bool`、`Char`、`String`
- 泛型类型：`Array<T>`
- 特殊类型：`Any`（动态）

---

## 4. 变量声明

```paw
let x: Int = 10
x = x + 1             # 赋值（可变）
```

---

## 5. 表达式

- 算术：`+` `-` `*` `/` `%`
- 比较：`==` `!=` `<` `<=` `>` `>=`
- 逻辑：`&&` `||` `!`
- 字符串拼接：`"Hi " + name + "!"`
- 括号改变优先级：`(a + b) * c`

---

## 6. 语句

- 赋值：`let` / 修改后直接 `=`
- 输出：`say <expr>`
- 输入：`ask <string literal>` 或 `let x: String <- ask "?"`
- 返回：`return <expr>` 或 `return`

---

## 7. 控制流

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

## 8. 函数

```paw
fun name(param1: Int, param2: Float): String {
  return "…"
}
let s: String = name(1, 2.5)
```

---

## 9. 数组

```paw
let a: Array<Int> = [1,2,3]
say a[0]        # 索引
say a.length    # 属性
```

---

## 10. 类型转换

显式转换使用 `as`：

```paw
let i: Int = 3
let f: Float = i as Float     # Int → Float
say f + 1.5
```

- 数值间（Int/Float）互转
- 同类型转换（如 String as String）等价于无操作
- 不兼容转换（String→Int）为编译错误

---

## 11. 注释

```paw
# 单行注释
let x: Int = 5   # 行尾注释
```

---

## 12. 异常处理

PawScript 的异常关键字：

| 关键字    | 功能                       |
|---------|--------------------------|
| `bark`    | 抛出异常（throw）            |
| `sniff`   | 尝试块（try）                 |
| `snatch`  | 捕获块（catch），绑定异常变量     |
| `lastly`  | 最终块（finally），无论是否异常都执行 |

### 抛出

```paw
bark "error message"
```

立即跳转至最近的 `snatch` 块。

### 捕获

```paw
sniff {
  …        # 尝试区
} snatch (e) {
  …        # 捕获区，e 是异常消息
} lastly {
  …        # 最终区
}
```

- 未抛出则跳过 `snatch`，仍执行 `lastly`
- 抛出后执行 `snatch`，再执行 `lastly`

---

## 13. 完整示例

```paw
# 计算倒数并处理除零

fun reciprocal(x: Int): Float {
    if x == 0 {
        bark "division by zero"      # 抛出
    }
    return 1.0 / (x as Float)        # 转型
}

sniff {
    say "Calling reciprocal(2)…"
    let a: Float = reciprocal(2)
    say "Result: " + a

    say "Calling reciprocal(0)…"
    let b: Float = reciprocal(0)    # 抛出→跳到 snatch
    say "Won’t run"
} snatch (err) {
    say "Caught error: " + err       # 捕获消息
} lastly {
    say "Cleanup done"
}

say "Done."
```

**输出：**

```
Calling reciprocal(2)…
Result: 0.5
Calling reciprocal(0)…
Caught error: division by zero
Cleanup done
Done.
```  

---

祝你编程愉快！  
更多扩展与自举，请见项目源码。