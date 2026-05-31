use hudhudscript_bytecode::Value16;
use hudhudscript_shared_builtins::xdg_ops::{
    xdg_cache_home, xdg_config_home, xdg_data_home, xdg_desktop_files, xdg_mime_type,
    xdg_parse_desktop, xdg_runtime_dir,
};

#[test]
fn test_data_home_default() {
    std::env::remove_var("XDG_DATA_HOME");
    let result = xdg_data_home(&[]).unwrap();
    if let Some(s) = result.as_str() {
        assert!(s.ends_with("/.local/share"));
    } else {
        panic!("Expected string");
    }
}

#[test]
fn test_data_home_env() {
    std::env::set_var("XDG_DATA_HOME", "/tmp/test_data");
    let result = xdg_data_home(&[]).unwrap();
    assert_eq!(result, Value16::string("/tmp/test_data".to_string()));
    std::env::remove_var("XDG_DATA_HOME");
}

#[test]
fn test_config_home_default() {
    std::env::remove_var("XDG_CONFIG_HOME");
    let result = xdg_config_home(&[]).unwrap();
    if let Some(s) = result.as_str() {
        assert!(s.ends_with("/.config"));
    } else {
        panic!("Expected string");
    }
}

#[test]
fn test_cache_home_default() {
    std::env::remove_var("XDG_CACHE_HOME");
    let result = xdg_cache_home(&[]).unwrap();
    if let Some(s) = result.as_str() {
        assert!(s.ends_with("/.cache"));
    } else {
        panic!("Expected string");
    }
}

#[test]
fn test_runtime_dir_unset() {
    std::env::remove_var("XDG_RUNTIME_DIR");
    let result = xdg_runtime_dir(&[]).unwrap();
    assert_eq!(result, Value16::null());
}

#[test]
fn test_runtime_dir_set() {
    std::env::set_var("XDG_RUNTIME_DIR", "/run/user/1000");
    let result = xdg_runtime_dir(&[]).unwrap();
    assert_eq!(result, Value16::string("/run/user/1000".to_string()));
    std::env::remove_var("XDG_RUNTIME_DIR");
}

#[test]
fn test_mime_type_known() {
    let result = xdg_mime_type(&[Value16::string("image.png".to_string())]).unwrap();
    assert_eq!(result, Value16::string("image/png".to_string()));
}

#[test]
fn test_mime_type_unknown() {
    let result = xdg_mime_type(&[Value16::string("file.xyz123".to_string())]).unwrap();
    assert_eq!(
        result,
        Value16::string("application/octet-stream".to_string())
    );
}

#[test]
fn test_mime_type_various() {
    let cases = vec![
        ("doc.pdf", "application/pdf"),
        ("style.css", "text/css"),
        ("video.mp4", "video/mp4"),
        ("song.mp3", "audio/mpeg"),
        ("data.json", "application/json"),
    ];
    for (file, expected) in cases {
        let result = xdg_mime_type(&[Value16::string(file.to_string())]).unwrap();
        assert_eq!(
            result,
            Value16::string(expected.to_string()),
            "Failed for {}",
            file
        );
    }
}

#[test]
fn test_parse_desktop_file() {
    // Create a temporary .desktop file
    let tmp = std::env::temp_dir().join("test_hudhud.desktop");
    std::fs::write(
        &tmp,
        "[Desktop Entry]\n\
         Type=Application\n\
         Name=Test App\n\
         Exec=/usr/bin/test-app %u\n\
         Icon=test-icon\n\
         Comment=A test application\n\
         Categories=Utility;Development;\n",
    )
    .unwrap();

    let result = xdg_parse_desktop(&[Value16::string(tmp.to_string_lossy().to_string())]).unwrap();
    if let Some(obj) = result.as_object() {
        assert_eq!(
            obj.get("Name"),
            Some(&Value16::string("Test App".to_string()))
        );
        assert_eq!(
            obj.get("Exec"),
            Some(&Value16::string("/usr/bin/test-app %u".to_string()))
        );
        assert_eq!(
            obj.get("Icon"),
            Some(&Value16::string("test-icon".to_string()))
        );
        assert_eq!(
            obj.get("Type"),
            Some(&Value16::string("Application".to_string()))
        );
        assert_eq!(
            obj.get("Comment"),
            Some(&Value16::string("A test application".to_string()))
        );
        assert_eq!(
            obj.get("Categories"),
            Some(&Value16::string("Utility;Development;".to_string()))
        );
    } else {
        panic!("Expected object");
    }

    let _ = std::fs::remove_file(&tmp);
}

#[test]
fn test_desktop_files_returns_array() {
    let result = xdg_desktop_files(&[]).unwrap();
    assert!(result.as_array().is_some());
}
