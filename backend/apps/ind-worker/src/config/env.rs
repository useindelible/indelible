pub(super) trait EnvSource {
    fn get(&self, key: &str) -> Option<String>;
}

pub(super) struct ProcessEnv;

impl EnvSource for ProcessEnv {
    fn get(&self, key: &str) -> Option<String> {
        std::env::var(key).ok()
    }
}
