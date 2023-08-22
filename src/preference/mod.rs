use lazy_static::lazy_static;
use preferences::{AppInfo, Preferences, PreferencesMap};
use std::{
    str::FromStr,
    sync::{Arc, RwLock},
};

const PREFS_PATH: &str = "planner";
const APP_INFO: AppInfo = AppInfo {
    name: "kelpie-flight-planner",
    author: "shartrec.com",
};

lazy_static! {
    static ref MANAGER: PreferenceManager = PreferenceManager {
        preferences: {
            match PreferencesMap::<String>::load(&APP_INFO, PREFS_PATH) {
                Ok(map) => Arc::new(RwLock::new(map)),
                Err(e) => {
                    println!("Error openning preferences {}", e);
                    Arc::new(RwLock::new(PreferencesMap::new()))
                }
            }
        },
        path: PREFS_PATH,
    };
}

pub struct PreferenceManager {
    preferences: Arc<RwLock<PreferencesMap>>,
    path: &'static str,
}

impl PreferenceManager {
    pub fn get<T: FromStr>(&self, key: &str) -> Option<T> {
        match self.preferences.read().unwrap().get(key) {
            Some(s) => match s.parse::<T>() {
                Ok(i) => Some(i),
                Err(_e) => None,
            },
            None => None,
        }
    }
    pub fn put<T: ToString>(&self, key: &str, value: T) -> () {
        {
            let mut prefs = self.preferences.write().unwrap();
            prefs.insert(key.to_string(), value.to_string());
        }
        self.store();
        ()
    }

    pub fn remove(&self, key: &str) -> () {
        {
            let mut prefs = self.preferences.write().unwrap();
            let _e = prefs.remove(key);
        }
        self.store();
        ()
    }

    pub fn clear(&self) -> () {
        {
            let mut prefs = self.preferences.write().unwrap();
            prefs.clear();
        }
        self.store();
        ()
    }

    fn store(&self) {
        let prefs = self.preferences.read().unwrap();
        let _ = prefs.save(&APP_INFO, self.path);
        ()
    }
}

pub fn manager() -> &'static PreferenceManager {
    &MANAGER
}

#[cfg(test)]
mod tests {
    use crate::preference;
    use preferences::PreferencesMap;
    use std::sync::{Arc, RwLock};

    #[test]
    fn test_save_restore() {
        let manager = preference::PreferenceManager {
            preferences: Arc::new(RwLock::new(PreferencesMap::new())),
            path: "kelpie-unit-test",
        };

        manager.put("Test_KEY 1", "First");
        manager.put("Test_KEY 2", 1 as i32);
        manager.put("Test_KEY 3", 24.66 as f64);

        assert_eq!(
            manager.get::<String>("Test_KEY 1"),
            Some("First".to_string())
        );
        assert_eq!(manager.get::<i32>("Test_KEY 2"), Some(1));
        assert_eq!(manager.get::<f64>("Test_KEY 3"), Some(24.66));
    }
}
