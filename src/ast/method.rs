/// 支持的所有方法
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Method {
    // String methods
    Trim,
    ToUppercase,
    ToLowercase,
    Length,
    StartsWith,
    EndsWith,
    Contains,
    // Array methods
    Push,
    Pop,
    LengthArr,   // 避免跟 String.length 冲突
    // …根据需要再加…
    Other,       // 用于模块成员调用或用户自定义
}

impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Method::Trim => write!(f, "trim"),
            Method::ToUppercase => write!(f, "to_uppercase"),
            Method::ToLowercase => write!(f, "to_lowercase"),
            Method::Length => write!(f, "length"),
            Method::StartsWith => write!(f, "starts_with"),
            Method::EndsWith => write!(f, "ends_with"),
            Method::Contains => write!(f, "contains"),
            Method::Push => write!(f, "push"),
            Method::Pop => write!(f, "pop"),
            Method::LengthArr => write!(f, "length"),
            _ => write!(f, "{:?}", self),
        }
    }
}

impl Method {
    pub fn as_str(&self) -> &str {
        match self {
            Method::Trim         => "trim",
            Method::ToUppercase  => "to_uppercase",
            Method::ToLowercase  => "to_lowercase",
            Method::Length       => "length",
            Method::StartsWith   => "starts_with",
            Method::EndsWith     => "ends_with",
            Method::Contains     => "contains",
            Method::Push         => "push",
            Method::Pop          => "pop",
            Method::LengthArr    => "length",
            Method::Other        => "", // or panic! if you never use Other here
        }
    }
}