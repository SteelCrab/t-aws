use crate::error::{AppError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Os {
    MacOS,
    Windows,
    Linux,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arch {
    X86_64,
    Arm64,
}

#[derive(Debug, Clone)]
pub struct Platform {
    pub os: Os,
    pub arch: Arch,
}

impl Platform {
    pub fn detect() -> Result<Self> {
        let os = Self::detect_os()?;
        let arch = Self::detect_arch()?;
        Ok(Self { os, arch })
    }

    fn detect_os() -> Result<Os> {
        match std::env::consts::OS {
            "macos" => Ok(Os::MacOS),
            "windows" => Ok(Os::Windows),
            "linux" => Ok(Os::Linux),
            other => Err(AppError::UnsupportedPlatform {
                os: other.to_string(),
                arch: std::env::consts::ARCH.to_string(),
            }),
        }
    }

    fn detect_arch() -> Result<Arch> {
        match std::env::consts::ARCH {
            "x86_64" => Ok(Arch::X86_64),
            "aarch64" => Ok(Arch::Arm64),
            other => Err(AppError::UnsupportedPlatform {
                os: std::env::consts::OS.to_string(),
                arch: other.to_string(),
            }),
        }
    }

    pub fn download_url(&self) -> &'static str {
        match self.os {
            Os::MacOS => "https://awscli.amazonaws.com/AWSCLIV2.pkg",
            Os::Windows => "https://awscli.amazonaws.com/AWSCLIV2.msi",
            Os::Linux => match self.arch {
                Arch::X86_64 => "https://awscli.amazonaws.com/awscli-exe-linux-x86_64.zip",
                Arch::Arm64 => "https://awscli.amazonaws.com/awscli-exe-linux-aarch64.zip",
            },
        }
    }

    pub fn installer_filename(&self) -> &'static str {
        match self.os {
            Os::MacOS => "AWSCLIV2.pkg",
            Os::Windows => "AWSCLIV2.msi",
            Os::Linux => "awscliv2.zip",
        }
    }

    pub fn display_name(&self) -> String {
        let os_name = match self.os {
            Os::MacOS => "macOS",
            Os::Windows => "Windows",
            Os::Linux => "Linux",
        };
        let arch_name = match self.arch {
            Arch::X86_64 => "x86_64",
            Arch::Arm64 => "arm64",
        };
        format!("{} ({})", os_name, arch_name)
    }
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}
