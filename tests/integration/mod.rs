//! Integration tests for Syntropy public API and CLI
//!
//! These tests verify behavior from an external user's perspective.

mod case_sensitivity_test;
mod circular_dependency_test;
mod cli_execute_test;
mod cli_init_test;
mod cli_list_test;
mod colors_loading_test;
mod config_validation_test;
mod exit_code_integration_test;
mod lua_expand_path_test;
mod lua_registry_cleanup_test;
mod lua_runtime_error_test;
mod malformed_module_test;
mod module_edge_cases_test;
mod module_nesting_and_merge_test;
mod multisource_execute_routing_test;
mod multisource_items_partial_failure_test;
mod multisource_partial_failure_test;
mod path_expansion_test;
mod plugin_function_type_validation_test;
mod plugin_isolation_test;
mod plugin_lib_isolation_test;
mod plugin_lib_loading_test;
mod plugin_loading_edge_cases_test;
mod plugin_loading_graceful_degradation_test;
mod plugin_loading_test;
mod plugin_manager_test;
mod plugin_validation_merge_test;
mod plugin_validation_test;
mod shared_modules_test;
mod signal_handling_test;
mod tag_stripping_execute_test;
