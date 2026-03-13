//! GitID Tauri Application — desktop GUI for managing Git identities.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;

use commands::*;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            // Profiles
            get_profiles,
            get_profile,
            create_profile,
            update_profile,
            delete_profile,
            // Rules
            get_rules,
            add_rule,
            remove_rule,
            set_default_profile,
            reorder_rules,
            // Status
            get_status,
            // Doctor
            run_doctor,
            install_credential_helper,
            // Repo detection
            scan_repos,
            // SSH
            test_ssh_connection,
            // Detection / import
            detect_setup,
            import_suggested_profile,
            // Guard
            guard_status,
            guard_install,
            guard_uninstall,
            guard_fix,
            // Suggestions / learning
            get_suggestions,
            get_activity_count,
            apply_suggestion,
            // CLI installation
            check_cli_installed,
            install_cli,
        ])
        .setup(|_app| {
            // System tray will be configured here
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running GitID");
}
