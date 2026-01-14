use std::path::Path;
use std::process::Command;

use crate::error::{AppError, Result};
use crate::platform::{Os, Platform};

pub fn install_aws_cli(platform: &Platform, installer_path: &Path) -> Result<()> {
    match platform.os {
        Os::MacOS => install_macos(installer_path),
        Os::Windows => install_windows(installer_path),
        Os::Linux => install_linux(installer_path),
    }
}

fn install_macos(pkg_path: &Path) -> Result<()> {
    let status = Command::new("sudo")
        .args(["installer", "-pkg"])
        .arg(pkg_path)
        .args(["-target", "/"])
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(AppError::InstallError {
            message: format!(
                "installer command failed with exit code: {:?}",
                status.code()
            ),
        })
    }
}

fn install_windows(msi_path: &Path) -> Result<()> {
    let status = Command::new("msiexec")
        .args(["/i"])
        .arg(msi_path)
        .args(["/quiet", "/norestart"])
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(AppError::InstallError {
            message: format!("msiexec command failed with exit code: {:?}", status.code()),
        })
    }
}

fn install_linux(zip_path: &Path) -> Result<()> {
    let temp_dir = zip_path.parent().unwrap_or(Path::new("/tmp"));
    let extract_dir = temp_dir.join("aws-cli-install");

    // Clean up previous extraction if exists
    if extract_dir.exists() {
        std::fs::remove_dir_all(&extract_dir)?;
    }
    std::fs::create_dir_all(&extract_dir)?;

    // Extract zip
    let file = std::fs::File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    archive.extract(&extract_dir)?;

    // Run install script
    let install_script = extract_dir.join("aws").join("install");
    let status = Command::new("sudo").arg(&install_script).status()?;

    // Cleanup
    let _ = std::fs::remove_dir_all(&extract_dir);

    if status.success() {
        Ok(())
    } else {
        Err(AppError::InstallError {
            message: format!(
                "AWS CLI install script failed with exit code: {:?}",
                status.code()
            ),
        })
    }
}

pub fn verify_installation() -> Result<String> {
    let output = Command::new("aws").arg("--version").output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(AppError::CommandError {
            command: "aws --version".to_string(),
            message: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }
}

pub fn uninstall_aws_cli(platform: &Platform) -> Result<()> {
    match platform.os {
        Os::MacOS => uninstall_macos(),
        Os::Windows => uninstall_windows(),
        Os::Linux => uninstall_linux(),
    }
}

fn uninstall_macos() -> Result<()> {
    // Remove AWS CLI symlinks and installation directory
    let commands = [
        ("sudo", vec!["rm", "-rf", "/usr/local/aws-cli"]),
        ("sudo", vec!["rm", "-f", "/usr/local/bin/aws"]),
        ("sudo", vec!["rm", "-f", "/usr/local/bin/aws_completer"]),
    ];

    for (cmd, args) in commands {
        let status = Command::new(cmd).args(&args).status()?;
        if !status.success() {
            return Err(AppError::InstallError {
                message: format!("Failed to execute: {} {:?}", cmd, args),
            });
        }
    }
    Ok(())
}

fn uninstall_windows() -> Result<()> {
    // Use wmic to find and uninstall AWS CLI
    let output = Command::new("wmic")
        .args([
            "product",
            "where",
            "name like '%AWS Command Line Interface%'",
            "call",
            "uninstall",
            "/nointeractive",
        ])
        .output()?;

    if output.status.success() {
        Ok(())
    } else {
        // Try alternative: uninstall via registry GUID
        let status = Command::new("msiexec")
            .args(["/x", "{AWS CLI}", "/quiet", "/norestart"])
            .status()?;

        if status.success() {
            Ok(())
        } else {
            Err(AppError::InstallError {
                message:
                    "Failed to uninstall AWS CLI. Please uninstall manually via Control Panel."
                        .to_string(),
            })
        }
    }
}

fn uninstall_linux() -> Result<()> {
    // Remove AWS CLI installation directory and symlinks
    let commands = [
        ("sudo", vec!["rm", "-rf", "/usr/local/aws-cli"]),
        ("sudo", vec!["rm", "-f", "/usr/local/bin/aws"]),
        ("sudo", vec!["rm", "-f", "/usr/local/bin/aws_completer"]),
    ];

    for (cmd, args) in commands {
        let status = Command::new(cmd).args(&args).status()?;
        if !status.success() {
            return Err(AppError::InstallError {
                message: format!("Failed to execute: {} {:?}", cmd, args),
            });
        }
    }
    Ok(())
}
