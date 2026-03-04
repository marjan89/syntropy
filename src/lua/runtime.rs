use anyhow::Result;
use mlua::{Lua, LuaOptions, StdLib, Table};

use crate::lua::stdlib::register_syntropy_stdlib;

pub const MERGE_LUA_FN_KEY: &str = "merge";
const MERGE_LUA: &str = r#"
-- Detects if a table is array-like (sequential integer keys starting at 1)
local function is_array(t)
    if type(t) ~= "table" then
        return false
    end
    local i = 0
    for _ in pairs(t) do
        i = i + 1
        if t[i] == nil then
            return false
        end
    end
    return true
end

-- Recursively merges two tables
-- override values take precedence over base values
function merge(base, override)
    -- If override is not a table, return it directly
    if type(override) ~= "table" then
        return override
    end

    -- If base is not a table, return override
    if type(base) ~= "table" then
        return override
    end

    -- If override is an array, replace it entirely (don't merge elements)
    if is_array(override) then
        return override
    end

    -- Merge maps/objects
    local result = {}

    -- Copy all base keys
    for k, v in pairs(base) do
        result[k] = v
    end

    -- Apply overrides (recursively for nested tables)
    for k, v in pairs(override) do
        if type(v) == "table" and type(result[k]) == "table" and not is_array(v) then
            -- Both are tables and override is not an array - recurse
            result[k] = merge(result[k], v)
        else
            -- Override value directly
            result[k] = v
        end
    end

    return result
end

return merge
"#;

pub fn create_lua_vm() -> Result<Lua> {
    let lua = Lua::new_with(StdLib::ALL_SAFE, LuaOptions::default())?;

    let os_table: Table = lua.globals().get("os")?;

    let blocked_exit = lua.create_function(|_, _: mlua::MultiValue| -> mlua::Result<()> {
        Err(mlua::Error::external(
            "os.exit is not available in the plugin sandbox",
        ))
    })?;
    os_table.set("exit", blocked_exit)?;

    let blocked_execute = lua.create_function(|_, _: mlua::MultiValue| -> mlua::Result<()> {
        Err(mlua::Error::external(
            "os.execute is not available in the plugin sandbox",
        ))
    })?;
    os_table.set("execute", blocked_execute)?;

    register_syntropy_stdlib(&lua)?;

    lua.globals().set("os", os_table)?;

    // Inject merge function for plugin override system
    let merge_fn: mlua::Function = lua.load(MERGE_LUA).eval()?;
    lua.globals().set("merge", merge_fn)?;

    // Suppress print() - plugins must not write directly to stdout
    let noop = lua.create_function(|_, _: mlua::MultiValue| Ok(()))?;
    lua.globals().set("print", noop)?;

    Ok(lua)
}
