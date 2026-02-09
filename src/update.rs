use self_update::backends::github::Update;
use self_update::cargo_crate_version;

const REPO_OWNER: &str = "SteelCrab";
const REPO_NAME: &str = "emd";

pub struct UpdateStatus {
    updated: bool,
    version: String,
}

impl UpdateStatus {
    #[cfg(test)]
    pub fn new(updated: bool, version: String) -> Self {
        Self { updated, version }
    }

    pub fn updated(&self) -> bool {
        self.updated
    }

    pub fn version(&self) -> &str {
        &self.version
    }
}

pub trait ReleaseUpdater {
    fn update(&self, current_version: &str) -> Result<UpdateStatus, Box<dyn std::error::Error>>;
}

pub struct GithubUpdater;

impl ReleaseUpdater for GithubUpdater {
    fn update(&self, current_version: &str) -> Result<UpdateStatus, Box<dyn std::error::Error>> {
        let status = Update::configure()
            .repo_owner(REPO_OWNER)
            .repo_name(REPO_NAME)
            .bin_name("emd")
            .show_download_progress(true)
            .current_version(current_version)
            .build()?
            .update()?;

        Ok(UpdateStatus {
            updated: status.updated(),
            version: status.version().to_string(),
        })
    }
}

pub fn perform_update() -> Result<(), Box<dyn std::error::Error>> {
    perform_update_internal(&GithubUpdater)
}

fn perform_update_internal<U: ReleaseUpdater>(
    updater: &U,
) -> Result<(), Box<dyn std::error::Error>> {
    let current_version = cargo_crate_version!();
    println!("Current version: v{}", current_version);
    println!("Checking for updates...");

    let status = updater.update(current_version)?;

    if status.updated() {
        println!(
            "Update complete: v{} -> v{}",
            current_version,
            status.version()
        );
    } else {
        println!("Already up to date: v{}", current_version);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockUpdater {
        should_update: bool,
        new_version: String,
        should_fail: bool,
    }

    impl MockUpdater {
        fn new(should_update: bool, new_version: &str) -> Self {
            Self {
                should_update,
                new_version: new_version.to_string(),
                should_fail: false,
            }
        }

        fn with_error() -> Self {
            Self {
                should_update: false,
                new_version: String::new(),
                should_fail: true,
            }
        }
    }

    impl ReleaseUpdater for MockUpdater {
        fn update(
            &self,
            _current_version: &str,
        ) -> Result<UpdateStatus, Box<dyn std::error::Error>> {
            if self.should_fail {
                return Err("Mock update failed".into());
            }
            Ok(UpdateStatus::new(
                self.should_update,
                self.new_version.clone(),
            ))
        }
    }

    #[test]
    fn test_perform_update_no_update() {
        let updater = MockUpdater::new(false, "0.1.0");
        let result = perform_update_internal(&updater);
        assert!(result.is_ok());
    }

    #[test]
    fn test_perform_update_with_update() {
        let updater = MockUpdater::new(true, "0.2.0");
        let result = perform_update_internal(&updater);
        assert!(result.is_ok());
    }

    #[test]
    fn test_perform_update_failure() {
        let updater = MockUpdater::with_error();
        let result = perform_update_internal(&updater);
        assert!(result.is_err());
        assert_eq!(result.err().unwrap().to_string(), "Mock update failed");
    }
}
