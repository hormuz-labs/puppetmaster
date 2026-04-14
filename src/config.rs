use std::env;

#[derive(Debug, Clone)]
pub enum AccessControl {
    All,
    Restricted(Vec<u64>),
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub allowed_users: AccessControl,
    pub unauthorized_message: String,
    pub allow_in_groups: bool,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let allowed_users_str = env::var("ALLOWED_USERS").unwrap_or_else(|_| "*".to_string());
        
        let allowed_users = if allowed_users_str.trim() == "*" {
            AccessControl::All
        } else {
            let ids: Vec<u64> = allowed_users_str
                .split(',')
                .filter_map(|s| s.trim().parse::<u64>().ok())
                .collect();
            AccessControl::Restricted(ids)
        };

        let unauthorized_message = env::var("UNAUTHORIZED_MESSAGE")
            .unwrap_or_else(|_| "🚫 You are not authorized to use this bot. Please contact the administrator or use @userinfobot to get your Telegram ID and add it to ALLOWED_USERS.".to_string());
        
        let allow_in_groups = env::var("ALLOW_IN_GROUPS")
            .unwrap_or_else(|_| "true".to_string())
            .to_lowercase() == "true";

        Self {
            allowed_users,
            unauthorized_message,
            allow_in_groups,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Mutex to prevent environment variable races during tests
    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn test_app_config_default() {
        let _lock = ENV_MUTEX.lock().unwrap();
        unsafe { env::remove_var("ALLOWED_USERS"); }
        unsafe { env::remove_var("ALLOW_IN_GROUPS"); }
        unsafe { env::remove_var("UNAUTHORIZED_MESSAGE"); }

        let config = AppConfig::from_env();
        assert!(matches!(config.allowed_users, AccessControl::All));
        assert!(config.allow_in_groups);
        assert!(config.unauthorized_message.contains("not authorized"));
    }

    #[test]
    fn test_app_config_restricted_users() {
        let _lock = ENV_MUTEX.lock().unwrap();
        unsafe { env::set_var("ALLOWED_USERS", " 1234 , 5678, invalid, 9012"); }
        let config = AppConfig::from_env();
        
        match config.allowed_users {
            AccessControl::Restricted(ids) => {
                assert_eq!(ids, vec![1234, 5678, 9012]);
            }
            _ => panic!("Expected restricted access control"),
        }
    }

    #[test]
    fn test_app_config_disallow_groups() {
        let _lock = ENV_MUTEX.lock().unwrap();
        unsafe { env::set_var("ALLOW_IN_GROUPS", "false"); }
        let config = AppConfig::from_env();
        assert!(!config.allow_in_groups);
        
        unsafe { env::set_var("ALLOW_IN_GROUPS", "False"); }
        let config = AppConfig::from_env();
        assert!(!config.allow_in_groups);
    }
}
