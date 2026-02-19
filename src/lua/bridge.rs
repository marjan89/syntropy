use anyhow::{Context, Result};
use mlua::{Function, Lua, Table, Value};

pub fn get_lua_function(lua: &Lua, path: &[&str]) -> Result<Function> {
    let mut current: Value = Value::Table(lua.globals());

    for (i, segment) in path.iter().enumerate() {
        let table = current.as_table().with_context(|| {
            format!("Expected table at path segment '{}' (index {})", segment, i)
        })?;

        current = table.get(*segment).with_context(|| {
            format!(
                "Failed to get '{}' at path {}",
                segment,
                path[..=i].join(".")
            )
        })?;
    }

    current
        .as_function()
        .cloned()
        .with_context(|| format!("Expected function at path {}", path.join(".")))
}

pub fn get_optional_lua_function(lua: &Lua, path: &[&str]) -> Result<Option<Function>> {
    let mut current: Value = Value::Table(lua.globals());

    for (i, segment) in path.iter().enumerate() {
        let table = current.as_table().with_context(|| {
            format!("Expected table at path segment '{}' (index {})", segment, i)
        })?;

        if i == path.len() - 1 {
            match table.get::<Value>(*segment) {
                Ok(Value::Nil) => return Ok(None),
                Ok(value) => current = value,
                Err(_) => return Ok(None),
            }
        } else {
            current = table.get(*segment).with_context(|| {
                format!(
                    "Failed to get '{}' at path {}",
                    segment,
                    path[..=i].join(".")
                )
            })?;
        }
    }

    match current.as_function() {
        Some(f) => Ok(Some(f.clone())),
        None => Ok(None),
    }
}

pub fn lua_table_to_vec_string(table: Table, function_key: &str) -> Result<Vec<String>> {
    let mut items = Vec::new();

    for pair in table.pairs::<usize, String>() {
        let (_, item) = pair.with_context(|| {
            format!(
                "Error reading table entry for lua function: {}",
                function_key
            )
        })?;
        items.push(item);
    }

    Ok(items)
}

pub fn vec_string_to_lua_table(lua: &Lua, items: &[String], function_key: &str) -> Result<Table> {
    let table = lua.create_table().with_context(|| {
        format!(
            "Failed to create Lua table for lua function: {}",
            function_key
        )
    })?;

    for (i, item) in items.iter().enumerate() {
        table.set(i + 1, item.as_str()).with_context(|| {
            format!(
                "Failed to set table entry for lua function {} at index {}",
                function_key, i
            )
        })?;
    }

    Ok(table)
}
