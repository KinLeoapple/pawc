# PawScript

PawScript 是一门简洁、可扩展、支持脚本化的 Kotlin-风格语言，当前已实现以下核心功能：

## 一、语言前端

- **词法分析 & 语法解析**  
  - 支持常见运算符（`+ - * / % == != < <= > >= && || !`）  
  - 字面量：整型、浮点、布尔、字符、字符串  
  - 标识符、关键字（`let`、`fun`、`if`、`else`、`loop`、`match`、`return`、`bark`、`say`、`ask`、`sniff`/`snatch`/`lastly`、`break`、`continue` 等）  
  - 模板字符串与转义字符  
  - 多种循环：无限循环、`while`、`loop x in expr`、`loop i,x in expr`、`loop x in start..end`  
  - 模式匹配表达式：  
    ```paw
    match expr {
      Pattern1 -> { … }
      Pattern2 -> { … }
      _        -> { … }
    }
    ```

- **抽象语法树 (AST)**  
  - `Declaration`：`Import`、接口 (`tail`)、记录 (`record`)、函数 (`fun`)  
  - `Statement`：变量声明、赋值、条件、循环、`break`/`continue`、`return`、日志 (`bark`)、输出 (`say`)、输入 (`ask`)、异常处理 (`sniff`/`snatch`/`lastly`)  
  - `Expr`：字面量、变量、二元/一元运算、数组、字段访问、函数/方法调用、`await`、`as`、`match`、块表达式 (`{ … }`)

## 二、模块与导入

- **标准库 & 项目模块**  
  - 从项目 `src/` 和标准库 `stdlib/` 自动扫描 `.paw` 文件  
  - 支持 `import module [as alias]`，仅在遇到 `import` 时注入模块符号  
  - 引用方式：  
  ```paw
  import utils as u
  let v = u.VERSION        # 访问常量
  let f = u.foo            # 取函数值
  let r = u.Pair(1,2)      # 调用记录构造
````

## 三、类型系统

* **静态类型检查**

    * 基本类型：`Int`、`Float`、`Bool`、`Char`、`String`、`Void`
    * 可空类型：`T?`
    * 数组类型：`T[]`
    * 模块类型：`Module(name)`
    * 泛型类型：`Generic(name, [args…])`，内置 `Future<T>`
    * **函数类型**： `(A,B,…)->R`，一等公民，可赋值、传参
    * **类型变量**：`<T>` 支持泛型函数与泛型注解
    * **模式匹配**：根据字面量、变量、构造器（记录）自动绑定并检查分支返回类型一致
    * **类型转换**：`expr as Type`，支持安全的基本转换与可空提升
    * **异步类型**：`async fun` 返回 `Future<T>`，`await` 解包装为 `T`

## 四、控制流

* **循环**

    * `loop { … }` 无限循环
    * `loop x in expr { … }` 迭代集合
    * `loop i,x in expr { … }` 索引迭代
    * `loop x in start..end { … }` 范围迭代
    * `while` 风格：`loop cond { … }`
    * 支持 `break` 和 `continue`，并检查是否在循环内使用

* **条件**

    * `if (cond) { … } else { … }`

* **模式匹配**

    * `match` 表达式分支可写为单表达式或块表达式，最后一条语句的值即分支结果

## 五、异常与 I/O

* **异常处理**

  ```paw
  sniff {
    …try block…
  } snatch (e) {
    …catch block…
  } lastly {
    …finally block…
  }
  ```
* **内置 I/O**

    * `say expr` —— 向控制台输出一行
    * `bark expr` —— 调试日志（不换行）
    * `ask "prompt"` —— 交互式输入，返回 `String`

## 六、下一步

* ✅ 解释器（运行时）
* ✅ 记录类型构造与模式匹配
* ✅ 异步执行（`async`/`await`）
* 🔜 标准库拓展（文件、网络、集合操作）
* 🔜 REPL 与脚本打包
* 🔜 性能优化与编译后端

---

PawScript 致力于「简单、灵活、可脚本化」，欢迎在此基础上继续扩展与改进！\`\`\`
