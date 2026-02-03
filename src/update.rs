use self_update::backends::github::Update;
use self_update::cargo_crate_version;

const REPO_OWNER: &str = "SteelCrab";
const REPO_NAME: &str = "emd";

pub fn perform_update() -> Result<(), Box<dyn std::error::Error>> {
    let current_version = cargo_crate_version!();
    println!("Current version: v{}", current_version);
    println!("Checking for updates...");

    let status = Update::configure()
        .repo_owner(REPO_OWNER)
        .repo_name(REPO_NAME)
        .bin_name("emd")
        .show_download_progress(true)
        .current_version(current_version)
        .build()?
        .update()?;

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
