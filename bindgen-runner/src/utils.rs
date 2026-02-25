mod common_generator;
mod extract_layout_assertions;
mod run_rustfmt;
mod bitfields;
mod clangs;

pub use common_generator::*;
pub use extract_layout_assertions::*;
pub use run_rustfmt::*;
pub use bitfields::*;
pub use clangs::*;

#[derive(Debug)]
pub struct Platform {
    pub triplet: &'static str,
    #[allow(dead_code)]
    pub cfg: &'static str,
}

impl Platform {
    pub fn mod_name(&self) -> String {
        self.triplet.replace("-", "_")
    }

    pub const _WIN64: Self = Self {
        triplet: "x86_64-pc-windows-msvc",
        cfg: "all(target_arch = \"x86_64\", windows)",
    };

    pub const TYPICAL: [Self; 6] = [
        Self {
            triplet: "x86_64-unknown-linux-gnu",
            cfg: "all(target_arch = \"x86_64\", unix)",
        },
        Self {
            triplet: "x86_64-pc-windows-msvc",
            cfg: "all(target_arch = \"x86_64\", windows)",
        },
        Self {
            triplet: "i686-pc-windows-msvc",
            cfg: "all(target_arch = \"x86\", windows)",
        },
        Self {
            triplet: "i686-unknown-linux-gnu",
            cfg: "all(target_arch = \"x86\", unix)",
        },
        Self {
            triplet: "aarch64-unknown-linux-gnu",
            cfg: "all(target_arch = \"aarch64\", unix)",
        },
        Self {
            triplet: "aarch64-linux-android",
            cfg: "all(target_arch = \"aarch64\", target_os = \"android\")",
        },
    ];
}
