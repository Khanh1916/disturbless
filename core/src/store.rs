//! JSON persistence with atomic writes (so a sudden shutdown cannot corrupt the file).
//! The data keeps the same JSON shape as the Electron version, so existing data keeps working.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value, json};

pub const POPULAR_SITES: [&str; 9] = [
    "youtube.com",
    "facebook.com",
    "tiktok.com",
    "instagram.com",
    "twitter.com",
    "x.com",
    "reddit.com",
    "netflix.com",
    "twitch.tv",
];

fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

fn default_profiles() -> Value {
    json!([
        {
            "id": new_id(), "name": "Học sâu 50/10", "mode": "apps", "workMin": 50, "breakMin": 10, "rounds": 2,
            "appMode": "allow",
            "allowedApps": ["chrome", "msedge", "firefox", "winword", "excel", "powerpnt", "onenote",
                "acrord32", "notepad", "code", "obsidian", "notion", "anki", "zotero", "claude"],
            "blockedApps": [], "blockedSites": POPULAR_SITES
        },
        {
            "id": new_id(), "name": "Pomodoro 25/5", "mode": "apps", "workMin": 25, "breakMin": 5, "rounds": 4,
            "appMode": "block", "allowedApps": [],
            "blockedApps": ["discord", "steam", "spotify", "telegram", "zalo"], "blockedSites": POPULAR_SITES
        },
        {
            "id": new_id(), "name": "Khoá toàn màn hình 30 phút", "mode": "fullscreen", "workMin": 30, "breakMin": 0,
            "rounds": 1, "appMode": "off", "allowedApps": [], "blockedApps": [], "blockedSites": []
        }
    ])
}

fn defaults() -> Map<String, Value> {
    let Value::Object(m) = json!({
        "profiles": default_profiles(),
        "settings": { "strictness": "normal", "autostart": false, "persona": "bff", "voice": true, "cheerEverySec": 240, "introSeen": false },
        "lastPlan": null,
        "history": [],
        "active": null
    }) else {
        unreachable!()
    };
    m
}

pub struct Store {
    file: PathBuf,
    pub data: Map<String, Value>,
}

impl Store {
    pub fn open(file: impl Into<PathBuf>) -> Self {
        let file = file.into();
        let mut data = defaults();
        if let Ok(text) = fs::read_to_string(&file)
            && let Ok(Value::Object(raw)) = serde_json::from_str::<Value>(&text)
        {
            let mut settings = data.get("settings").cloned().unwrap_or(Value::Null);
            for (k, v) in raw {
                if k == "settings" {
                    if let (Some(base), Value::Object(over)) = (settings.as_object_mut(), v) {
                        base.extend(over);
                    }
                } else {
                    data.insert(k, v);
                }
            }
            data.insert("settings".into(), settings);
        }
        Self { file, data }
    }

    pub fn path(&self) -> &Path {
        &self.file
    }

    pub fn save(&self) -> io::Result<()> {
        if let Some(dir) = self.file.parent() {
            fs::create_dir_all(dir)?;
        }
        let tmp = self.file.with_extension("json.tmp");
        let text = serde_json::to_string_pretty(&Value::Object(self.data.clone()))
            .map_err(io::Error::other)?;
        fs::write(&tmp, text)?;
        fs::rename(&tmp, &self.file)
    }

    pub fn settings(&self) -> &Map<String, Value> {
        self.data["settings"]
            .as_object()
            .expect("settings is an object")
    }

    pub fn settings_mut(&mut self) -> &mut Map<String, Value> {
        self.data
            .get_mut("settings")
            .and_then(Value::as_object_mut)
            .expect("settings is an object")
    }

    pub fn array_mut(&mut self, key: &str) -> &mut Vec<Value> {
        let slot = self
            .data
            .entry(key.to_string())
            .or_insert_with(|| json!([]));
        if !slot.is_array() {
            *slot = json!([]);
        }
        slot.as_array_mut().expect("just made an array")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> PathBuf {
        let d =
            std::env::temp_dir().join(format!("disturbless-test-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        d.join("disturbless.json")
    }

    #[test]
    fn first_run_gives_defaults_and_save_round_trips() {
        let f = temp("a");
        let mut s = Store::open(&f);
        assert_eq!(s.data["profiles"].as_array().unwrap().len(), 3);
        assert_eq!(s.settings()["strictness"], "normal");
        s.settings_mut().insert("strictness".into(), json!("hard"));
        s.array_mut("history").push(json!({ "id": "x" }));
        s.save().unwrap();
        assert!(
            !f.with_extension("json.tmp").exists(),
            "temp file is renamed away"
        );
        let again = Store::open(&f);
        assert_eq!(again.settings()["strictness"], "hard");
        assert_eq!(
            again.settings()["voice"],
            true,
            "missing settings keep their defaults"
        );
        assert_eq!(again.data["history"][0]["id"], "x");
        let _ = fs::remove_dir_all(f.parent().unwrap());
    }

    #[test]
    fn a_corrupt_file_falls_back_to_defaults() {
        let f = temp("b");
        fs::create_dir_all(f.parent().unwrap()).unwrap();
        fs::write(&f, "{ not json").unwrap();
        let s = Store::open(&f);
        assert_eq!(s.data["profiles"].as_array().unwrap().len(), 3);
        let _ = fs::remove_dir_all(f.parent().unwrap());
    }

    #[test]
    fn reads_data_written_by_the_electron_version() {
        let f = temp("c");
        fs::create_dir_all(f.parent().unwrap()).unwrap();
        fs::write(&f, r#"{"settings":{"strictness":"easy","introSeen":true},"history":[{"completed":true}],"active":null,"lastPlan":null,"profiles":[]}"#).unwrap();
        let s = Store::open(&f);
        assert_eq!(s.settings()["strictness"], "easy");
        assert_eq!(s.settings()["cheerEverySec"], 240);
        assert_eq!(s.data["profiles"].as_array().unwrap().len(), 0);
        let _ = fs::remove_dir_all(f.parent().unwrap());
    }
}
