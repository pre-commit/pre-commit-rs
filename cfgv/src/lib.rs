use serde::Deserialize;
use serde_yaml::Value;
use std::fs;

pub trait Cfgv {
    fn cfgv_validate(ctx: &mut Vec<String>, v: &Value) -> anyhow::Result<Self>
    where
        Self: Sized;
}

pub fn type_name(v: &Value) -> String {
    match v {
        Value::Null => "null".into(),
        Value::Bool(_) => "bool".into(),
        Value::String(_) => "str".into(),
        Value::Number(n) => {
            if n.is_i64() || n.is_u64() {
                "int".into()
            } else {
                "float".into()
            }
        }
        Value::Sequence(_) => "list".into(),
        Value::Mapping(_) => "dict".into(),
        Value::Tagged(tag) => format!("{} value", tag.tag),
    }
}

pub fn ctx_s<S: std::fmt::Display>(ctx: &[String], msg: S) -> String {
    let start = ctx
        .iter()
        .map(|s| format!("==> {s}"))
        .collect::<Vec<String>>()
        .join("\n");
    format!("\n{start}\n=====> {msg}")
}

impl Cfgv for bool {
    fn cfgv_validate(ctx: &mut Vec<String>, v: &Value) -> anyhow::Result<Self> {
        if let Value::Bool(n) = v {
            Ok(*n)
        } else {
            anyhow::bail!(ctx_s(ctx, format!("Expected bool, got {}", type_name(v))))
        }
    }
}

impl Cfgv for i64 {
    fn cfgv_validate(ctx: &mut Vec<String>, v: &Value) -> anyhow::Result<Self> {
        if let Value::Number(n) = v {
            n.as_i64().ok_or_else(|| {
                anyhow::anyhow!(ctx_s(ctx, format!("Expected int, got {}", type_name(v))))
            })
        } else {
            anyhow::bail!(ctx_s(ctx, format!("Expected int, got {}", type_name(v))))
        }
    }
}

impl Cfgv for String {
    fn cfgv_validate(ctx: &mut Vec<String>, v: &Value) -> anyhow::Result<Self> {
        if let Value::String(s) = v {
            Ok(s.clone())
        } else {
            anyhow::bail!(ctx_s(ctx, format!("Expected str, got {}", type_name(v))));
        }
    }
}

impl<T: Cfgv> Cfgv for Vec<T> {
    fn cfgv_validate(ctx: &mut Vec<String>, v: &Value) -> anyhow::Result<Self> {
        if let Value::Sequence(lst) = v {
            let mut ret: Vec<T> = Vec::new();

            for (i, val) in lst.iter().enumerate() {
                ctx.push(format!("At index {i}"));
                ret.push(T::cfgv_validate(ctx, val)?);
                ctx.pop();
            }

            Ok(ret)
        } else {
            anyhow::bail!(ctx_s(ctx, format!("Expected list, got {}", type_name(v))))
        }
    }
}

pub fn parse<T: Cfgv>(v: &Value) -> anyhow::Result<T> {
    T::cfgv_validate(&mut Vec::new(), v)
}

pub fn load_file<T: Cfgv>(f: &str) -> anyhow::Result<T> {
    let mut ctx = vec![format!("File {f}")];
    let contents = match fs::read_to_string(f) {
        Ok(contents) => contents,
        Err(e) => anyhow::bail!(ctx_s(&ctx, e)),
    };
    let de = serde_yaml::Deserializer::from_str(&contents);
    let value = match serde_yaml::Value::deserialize(de) {
        Ok(value) => value,
        Err(e) => anyhow::bail!(ctx_s(&ctx, e)),
    };
    T::cfgv_validate(&mut ctx, &value)
}
