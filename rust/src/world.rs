use crate::scene::{load_scenes, SceneData};
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;
use tokio::sync::broadcast;

pub const DEFAULT_SCENE: &str = "scene.json";

/// The canvas is 384x256px (Game.map_grid: 24x16 tiles of 16px), so valid
/// player pixel coordinates run 0..=383 horizontally and 0..=255 vertically.
/// The original Scala bounds check let x go up to 384 inclusive, one past
/// the canvas edge; this fixes that off-by-one.
pub const MAX_X: i32 = 383;
pub const MAX_Y: i32 = 255;

/// Transition directions, matching the index order already used by the
/// existing scene JSON files' `transitions` arrays: [up, down, left, right].
pub const DIR_UP: usize = 0;
pub const DIR_DOWN: usize = 1;
pub const DIR_LEFT: usize = 2;
pub const DIR_RIGHT: usize = 3;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum OutEvent {
    #[serde(rename = "userMove")]
    UserMove {
        id: String,
        name: String,
        x: i32,
        y: i32,
    },
}

pub struct SceneEntry {
    pub data: SceneData,
    pub tx: broadcast::Sender<(String, OutEvent)>,
}

pub struct World {
    pub scenes: HashMap<String, SceneEntry>,
}

impl World {
    pub fn load(dir: &Path) -> Self {
        let raw = load_scenes(dir);
        if raw.is_empty() {
            tracing::warn!("no scenes loaded from {:?}; the game world is empty", dir);
        }

        let scenes = raw
            .into_iter()
            .map(|(name, data)| {
                let (tx, _rx) = broadcast::channel(256);
                (name, SceneEntry { data, tx })
            })
            .collect();

        let world = World { scenes };
        if !world.scenes.contains_key(DEFAULT_SCENE) {
            tracing::warn!(
                "default scene {} not found; falling back to an arbitrary loaded scene",
                DEFAULT_SCENE
            );
        }
        world
    }

    pub fn default_scene_name(&self) -> String {
        if self.scenes.contains_key(DEFAULT_SCENE) {
            DEFAULT_SCENE.to_string()
        } else {
            self.scenes.keys().next().cloned().unwrap_or_default()
        }
    }

    /// Resolves the scene a player lands in after crossing an edge. Falls
    /// back to the default scene if the current scene, the transition slot,
    /// or the named target scene doesn't exist -- mirroring the
    /// `.getOrElse { defaultSceneActor }` fallback in the original
    /// `WorldActor`, which several existing scene files' dangling
    /// transitions (e.g. `scene50.json`, never present on disk) rely on.
    pub fn transition_target(&self, current: &str, direction: usize) -> String {
        let fallback = self.default_scene_name();
        let Some(entry) = self.scenes.get(current) else {
            return fallback;
        };
        match entry.data.transitions.get(direction) {
            Some(target) if self.scenes.contains_key(target) => target.clone(),
            _ => fallback,
        }
    }
}
