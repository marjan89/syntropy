use mlua::Table;

use crate::{
    execution::SharedLua,
    lua::{
        get_lua_function, get_optional_lua_function, lua_table_to_vec_string,
        vec_string_to_lua_table,
    },
    plugins::{ItemSource, Plugin, Task},
};
use anyhow::{Context, Result};

pub async fn has_item_source_execute(lua: &SharedLua, task: &Task, source_key: &str) -> bool {
    let lua_guard = lua.lock().await;

    let path = &[
        &task.plugin_name,
        Plugin::LUA_PROPERTY_TASKS,
        &task.task_key,
        Task::LUA_PROPERTY_ITEM_SOURCES,
        source_key,
        ItemSource::LUA_FN_NAME_EXECUTE,
    ];

    get_optional_lua_function(&lua_guard, path)
        .ok()
        .flatten()
        .is_some()
}

pub async fn call_item_source_items(
    lua: &SharedLua,
    plugin_name: &str,
    task_key: &str,
    source_key: &str,
) -> Result<Vec<String>> {
    let lua_guard = lua.lock().await;

    let path = &[
        plugin_name,
        Plugin::LUA_PROPERTY_TASKS,
        task_key,
        Task::LUA_PROPERTY_ITEM_SOURCES,
        source_key,
        ItemSource::LUA_FN_NAME_ITEMS,
    ];
    let items_fn = get_lua_function(&lua_guard, path)?;

    // Set current plugin context for expand_path
    lua_guard
        .set_named_registry_value("__syntropy_current_plugin__", plugin_name)
        .context("Failed to set current plugin context")?;

    let result: Result<Table> = items_fn
        .call_async(())
        .await
        .with_context(|| format!("Error calling {}()", path.join(".")));

    // Clear plugin context
    lua_guard
        .set_named_registry_value("__syntropy_current_plugin__", mlua::Value::Nil)
        .context("Failed to clear current plugin context")?;

    let result = result?;
    lua_table_to_vec_string(result, ItemSource::LUA_FN_NAME_ITEMS)
}

pub async fn call_item_source_preselected_items(
    lua: &SharedLua,
    plugin_name: &str,
    task_key: &str,
    source_key: &str,
) -> Result<Vec<String>> {
    let lua_guard = lua.lock().await;

    let path = &[
        plugin_name,
        Plugin::LUA_PROPERTY_TASKS,
        task_key,
        Task::LUA_PROPERTY_ITEM_SOURCES,
        source_key,
        ItemSource::LUA_FN_NAME_PRESELECTED_ITEMS,
    ];

    // Set current plugin context
    lua_guard.set_named_registry_value("__syntropy_current_plugin__", plugin_name)?;

    let result = match get_optional_lua_function(&lua_guard, path)? {
        Some(func) => {
            let table_result: Result<Table> = func
                .call_async(())
                .await
                .with_context(|| format!("Error calling {}()", path.join(".")));
            match table_result {
                Ok(table) => {
                    lua_table_to_vec_string(table, ItemSource::LUA_FN_NAME_PRESELECTED_ITEMS)
                }
                Err(e) => Err(e),
            }
        }
        None => Ok(Vec::new()),
    };

    // Clear plugin context
    lua_guard.set_named_registry_value("__syntropy_current_plugin__", mlua::Value::Nil)?;

    result
}

pub async fn call_item_source_preview(
    lua: &SharedLua,
    plugin_name: &str,
    task_key: &str,
    source_key: &str,
    current_item: &str,
) -> Result<Option<String>> {
    let lua_guard = lua.lock().await;

    let path = &[
        plugin_name,
        Plugin::LUA_PROPERTY_TASKS,
        task_key,
        Task::LUA_PROPERTY_ITEM_SOURCES,
        source_key,
        ItemSource::LUA_FN_NAME_PREVIEW,
    ];

    lua_guard.set_named_registry_value("__syntropy_current_plugin__", plugin_name)?;

    let result = match get_optional_lua_function(&lua_guard, path)? {
        Some(func) => {
            let res: Result<String> = func
                .call_async(current_item)
                .await
                .with_context(|| format!("Error calling {}()", path.join(".")));
            match res {
                Ok(s) => Ok(Some(s)),
                Err(e) => Err(e),
            }
        }
        None => Ok(None),
    };

    lua_guard.set_named_registry_value("__syntropy_current_plugin__", mlua::Value::Nil)?;
    result
}

pub async fn call_item_source_execute(
    lua: &SharedLua,
    task: &Task,
    source_key: &str,
    selected_items: &[String],
) -> Result<(String, i32)> {
    let lua_guard = lua.lock().await;

    let path = &[
        &task.plugin_name,
        Plugin::LUA_PROPERTY_TASKS,
        &task.task_key,
        Task::LUA_PROPERTY_ITEM_SOURCES,
        source_key,
        ItemSource::LUA_FN_NAME_EXECUTE,
    ];

    lua_guard.set_named_registry_value("__syntropy_current_plugin__", task.plugin_name.as_str())?;

    let execute_fn = get_lua_function(&lua_guard, path)?;
    let items_table =
        vec_string_to_lua_table(&lua_guard, selected_items, ItemSource::LUA_FN_NAME_EXECUTE)?;

    let result: Result<(String, i32)> = execute_fn
        .call_async(items_table)
        .await
        .with_context(|| format!("Error calling {}(),", path.join(".")));

    lua_guard.set_named_registry_value("__syntropy_current_plugin__", mlua::Value::Nil)?;
    result
}

pub async fn call_task_pre_run(lua: &SharedLua, plugin_name: &str, task_key: &str) -> Result<()> {
    let lua_guard = lua.lock().await;

    let path = &[
        plugin_name,
        Plugin::LUA_PROPERTY_TASKS,
        task_key,
        Task::LUA_FN_NAME_PRE_RUN,
    ];

    lua_guard.set_named_registry_value("__syntropy_current_plugin__", plugin_name)?;

    let result = match get_optional_lua_function(&lua_guard, path)? {
        Some(func) => func
            .call_async::<()>(())
            .await
            .with_context(|| format!("Error calling {}()", path.join("."))),
        None => Ok(()),
    };

    lua_guard.set_named_registry_value("__syntropy_current_plugin__", mlua::Value::Nil)?;
    result
}

pub async fn call_task_post_run(lua: &SharedLua, plugin_name: &str, task_key: &str) -> Result<()> {
    let lua_guard = lua.lock().await;

    let path = &[
        plugin_name,
        Plugin::LUA_PROPERTY_TASKS,
        task_key,
        Task::LUA_FN_NAME_POST_RUN,
    ];

    lua_guard.set_named_registry_value("__syntropy_current_plugin__", plugin_name)?;

    let result = match get_optional_lua_function(&lua_guard, path)? {
        Some(func) => func
            .call_async::<()>(())
            .await
            .with_context(|| format!("Error calling {}()", path.join("."))),
        None => Ok(()),
    };

    lua_guard.set_named_registry_value("__syntropy_current_plugin__", mlua::Value::Nil)?;
    result
}

pub async fn call_task_preview(
    lua: &SharedLua,
    plugin_name: &str,
    task_key: &str,
    current_item: &str,
) -> Result<Option<String>> {
    let lua_guard = lua.lock().await;

    let path = &[
        plugin_name,
        Plugin::LUA_PROPERTY_TASKS,
        task_key,
        Task::LUA_FN_NAME_PREVIEW,
    ];

    lua_guard.set_named_registry_value("__syntropy_current_plugin__", plugin_name)?;

    let result = match get_optional_lua_function(&lua_guard, path)? {
        Some(func) => {
            let res: Result<String> = func
                .call_async(current_item)
                .await
                .with_context(|| format!("Error calling {}()", path.join(".")));
            match res {
                Ok(s) => Ok(Some(s)),
                Err(e) => Err(e),
            }
        }
        None => Ok(None),
    };

    lua_guard.set_named_registry_value("__syntropy_current_plugin__", mlua::Value::Nil)?;
    result
}

pub async fn call_task_execute(
    lua: &SharedLua,
    task: &Task,
    selected_items: &[String],
) -> Result<(String, i32)> {
    let lua_guard = lua.lock().await;

    let path = &[
        &task.plugin_name,
        Plugin::LUA_PROPERTY_TASKS,
        &task.task_key,
        Task::LUA_FN_NAME_EXECUTE,
    ];

    lua_guard.set_named_registry_value("__syntropy_current_plugin__", task.plugin_name.as_str())?;

    let execute_fn = get_lua_function(&lua_guard, path)?;
    let items_table =
        vec_string_to_lua_table(&lua_guard, selected_items, Task::LUA_FN_NAME_EXECUTE)?;

    let result: Result<(String, i32)> = execute_fn
        .call_async(items_table)
        .await
        .with_context(|| format!("Error calling {}()", path.join(".")));

    lua_guard.set_named_registry_value("__syntropy_current_plugin__", mlua::Value::Nil)?;
    result
}
