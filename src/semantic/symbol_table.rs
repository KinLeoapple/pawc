use ahash::AHashMap;
use crate::ast::ast::*;

#[derive(Debug, Clone)]
pub enum SymbolEntry<'a> {
    Variable {
        name: IdentifierNode<'a>,
        type_name: TypeNameNode<'a>,
    },
    Function {
        name: IdentifierNode<'a>,
        params: Vec<(IdentifierNode<'a>, TypeNameNode<'a>)>,
        return_type: Option<TypeNameNode<'a>>,
    },
    Record {
        name: IdentifierNode<'a>,
        fields: Vec<(IdentifierNode<'a>, TypeNameNode<'a>)>,
        methods: Vec<FunctionDefinitionNode<'a>>,
    },
    Protocol {
        name: IdentifierNode<'a>,
        methods: Vec<FunctionSignatureNode<'a>>,
    },
    Module {
        name: IdentifierNode<'a>,
        table: SymbolTable<'a>,
    },
}

/// 支持多层作用域的符号表
#[derive(Clone, Debug)]
pub struct SymbolTable<'a> {
    scopes: Vec<AHashMap<&'a str, SymbolEntry<'a>>>,
}

impl<'a> SymbolTable<'a> {
    /// 创建一个新的全局符号表
    pub fn new() -> Self {
        SymbolTable { scopes: vec![AHashMap::new()] }
    }

    /// 进入一个新作用域
    pub fn enter_scope(&mut self) {
        self.scopes.push(AHashMap::new());
    }

    /// 退出当前作用域
    pub fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    /// 在当前作用域插入一个符号，若重复则返回 Err
    pub fn insert(&mut self, entry: SymbolEntry<'a>) -> Result<(), String> {
        let name = match &entry {
            SymbolEntry::Variable { name, .. } => &name.name,
            SymbolEntry::Function { name, .. } => &name.name,
            SymbolEntry::Record { name, .. } => &name.name,
            SymbolEntry::Protocol { name, .. } => &name.name,
            SymbolEntry::Module { name, .. } => &name.name, 
        };
        let current = self.scopes.last_mut().unwrap();
        if current.contains_key(*name) {
            Err(format!("Symbol '{}' already defined in this scope", name))
        } else {
            current.insert(name, entry);
            Ok(())
        }
    }

    /// 按照作用域从内到外查找符号
    pub fn lookup(&self, name: &str) -> Option<&SymbolEntry<'a>> {
        for scope in self.scopes.iter().rev() {
            if let Some(entry) = scope.get(name) {
                return Some(entry);
            }
        }
        None
    }
}

/// 管理所有模块符号表的注册表
pub struct ModuleRegistry<'a> {
    tables: AHashMap<&'a str, SymbolTable<'a>>,
}

impl<'a> ModuleRegistry<'a> {
    /// 新建空注册表
    pub fn new() -> Self {
        ModuleRegistry { tables: AHashMap::new() }
    }

    /// 加载并解析模块，插入到注册表
    pub fn load_module(&mut self, module_name: &'a str, path: &str) -> Result<(), String> {
        // 防止重复注册
        if self.tables.contains_key(module_name) {
            return Err(format!("Module '{}' is already registered", module_name));
        }
        // TODO: 解析模块文件 path 并构建其符号表
        let table = SymbolTable::new();
        self.tables.insert(module_name, table);
        Ok(())
    }

    /// 获取某个模块的符号表
    pub fn lookup_module(&self, module_name: &str) -> Option<&SymbolTable<'a>> {
        self.tables.get(module_name)
    }

    /// 在指定模块中查找符号
    pub fn lookup_in_module(&self, module_name: &str, name: &str) -> Option<&SymbolEntry<'a>> {
        self.lookup_module(module_name)?.lookup(name)
    }
}