use crate::resources::LevelData;

pub fn save_level_data(level_data: &LevelData) {
    let json = serde_json::to_string_pretty(level_data).unwrap();
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            let _ = window.local_storage().ok().flatten()
                .map(|s| s.set_item("bakery_level", &json));
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = std::fs::write("bakery_level.json", &json);
    }
}

pub fn load_level_data() -> Option<LevelData> {
    #[cfg(target_arch = "wasm32")]
    {
        let json = web_sys::window()?
            .local_storage().ok()??
            .get_item("bakery_level").ok()??;
        serde_json::from_str(&json).ok()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let json = std::fs::read_to_string("bakery_level.json").ok()?;
        serde_json::from_str(&json).ok()
    }
}

pub fn delete_saved_level() {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            let _ = window.local_storage().ok().flatten()
                .map(|s| s.remove_item("bakery_level"));
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = std::fs::remove_file("bakery_level.json");
    }
}
