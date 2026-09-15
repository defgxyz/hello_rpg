use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct SceneData {
    pub scene: Vec<Vec<i32>>,
    pub transitions: Vec<String>,
}

/// Loads every `*.json` scene file in `dir`, keyed by file name (matching the
/// original Scala `Utils.loadWorld`'s convention so `transitions` entries and
/// `DEFAULT_SCENE` can reference scenes by file name).
///
/// Unlike the original, a malformed or unreadable scene file is logged and
/// skipped rather than crashing the whole server on startup.
pub fn load_scenes(dir: &Path) -> HashMap<String, SceneData> {
    let mut scenes = HashMap::new();

    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(err) => {
            tracing::error!("could not read scenes directory {:?}: {}", dir, err);
            return scenes;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let name = name.to_string();

        let contents = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(err) => {
                tracing::warn!("skipping scene {}: {}", name, err);
                continue;
            }
        };

        match serde_json::from_str::<SceneData>(&contents) {
            Ok(data) => {
                scenes.insert(name, data);
            }
            Err(err) => {
                tracing::warn!("skipping scene {}: invalid json: {}", name, err);
            }
        }
    }

    scenes
}
