use serde_json::Value;
use std::fs;
use std::path::PathBuf;

fn claude_dir() -> PathBuf {
    let home = std::env::var("HOME").expect("HOME not set");
    PathBuf::from(home).join(".claude")
}

fn binary_dest() -> PathBuf {
    claude_dir().join("claude-status")
}

fn settings_path() -> PathBuf {
    claude_dir().join("settings.json")
}

fn backup_path() -> PathBuf {
    claude_dir().join("statusline-backup.json")
}

fn read_settings() -> Value {
    let path = settings_path();
    if path.exists() {
        fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_else(|| serde_json::json!({}))
    } else {
        serde_json::json!({})
    }
}

fn write_settings(val: &Value) {
    let path = settings_path();
    let _ = fs::create_dir_all(path.parent().unwrap());
    let json = serde_json::to_string_pretty(val).unwrap();
    fs::write(path, json).expect("Failed to write settings.json");
}

pub fn install() {
    let dest = binary_dest();
    let dir = claude_dir();
    let _ = fs::create_dir_all(&dir);

    // Copy current binary to ~/.claude/claude-status
    let current_exe = std::env::current_exe().expect("Cannot determine current binary path");
    if current_exe != dest {
        fs::copy(&current_exe, &dest).expect("Failed to copy binary");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&dest, fs::Permissions::from_mode(0o755));
        }
    }

    // Backup existing statusLine config
    let mut settings = read_settings();
    if let Some(existing) = settings.get("statusLine") {
        let backup = serde_json::json!({ "statusLine": existing.clone() });
        let backup_json = serde_json::to_string_pretty(&backup).unwrap();
        fs::write(backup_path(), backup_json).expect("Failed to write backup");
        println!(
            "Backed up existing statusLine config to {:?}",
            backup_path()
        );
    }

    // Write new statusLine config
    let command = dest.to_string_lossy().to_string();
    settings["statusLine"] = serde_json::json!({
        "type": "command",
        "command": command
    });
    write_settings(&settings);

    println!("Installed claude-status to {}", dest.display());
    println!("Status line configured in {}", settings_path().display());
}

pub fn uninstall() {
    let dest = binary_dest();
    let bp = backup_path();
    let settings = read_settings();
    let has_status_line = settings.get("statusLine").is_some();

    // Check if there's anything to uninstall
    if !dest.exists() && !bp.exists() && !has_status_line {
        println!("Nothing to uninstall");
        return;
    }

    // Restore backup if exists
    let mut settings = settings;
    if bp.exists() {
        if let Ok(content) = fs::read_to_string(&bp)
            && let Ok(backup) = serde_json::from_str::<Value>(&content)
            && let Some(sl) = backup.get("statusLine")
        {
            settings["statusLine"] = sl.clone();
            write_settings(&settings);
            println!("Restored previous statusLine config");
        }
        let _ = fs::remove_file(&bp);
    } else if has_status_line {
        if let Some(obj) = settings.as_object_mut() {
            obj.remove("statusLine");
        }
        write_settings(&settings);
        println!("Removed statusLine config from settings.json");
    }

    // Remove binary
    if dest.exists() {
        let _ = fs::remove_file(&dest);
        println!("Removed {}", dest.display());
    }
}
