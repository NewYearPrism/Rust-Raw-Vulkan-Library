use std::fs;

fn main() {
    // for example, "x86_64-unknown-linux-gnu" target emits "x86_64_unknown_linux_gnu.rs"
    fn generate_platform(platform: &str) -> String {
        bindgen::builder()
            .clang_arg("-I../Vulkan-Headers/include")
            .clang_arg("-DVK_NO_PROTOTYPES")
            .clang_arg(format!("--target={platform}"))
            .clang_arg("-fparse-all-comments")
            .generate_comments(true)
            .header("../Vulkan-Headers/include/vulkan/vulkan.h")
            .allowlist_recursively(true)
            .allowlist_file(".*vulkan_core.h")
            .use_core()
            .raw_line(r"#![allow(non_snake_case)]")
            .raw_line(r"#![allow(non_camel_case_types)]")
            .raw_line(r"#![allow(non_upper_case_globals)]")
            .raw_line(r"#![allow(dead_code)]")
            .raw_line(r"#![allow(unused_imports)]")
            .raw_line(r"#![allow(clippy::useless_transmute)]")
            .raw_line(r"#![allow(clippy::too_many_arguments)]")
            .raw_line(r"#![allow(clippy::ptr_offset_with_cast)]")
            .sort_semantically(true)
            .generate()
            .unwrap()
            .to_string()
    }

    let platforms = vec![
        Platform {
            platform: "x86_64-unknown-linux-gnu",
            cfg_pred: "all(target_arch = \"x86_64\", unix)",
        },
        Platform {
            platform: "x86_64-pc-windows-msvc",
            cfg_pred: "all(target_arch = \"x86_64\", windows)",
        },
        Platform {
            platform: "i686-pc-windows-msvc",
            cfg_pred: "all(target_arch = \"x86\", windows)",
        },
        Platform {
            platform: "i686-unknown-linux-gnu",
            cfg_pred: "all(target_arch = \"x86\", unix)",
        },
        Platform {
            platform: "aarch64-unknown-linux-gnu",
            cfg_pred: "all(target_arch = \"aarch64\", unix)",
        },
        Platform {
            platform: "aarch64-linux-android",
            cfg_pred: "all(target_arch = \"aarch64\", target_os = \"android\")",
        },
    ];
    let mut lib_rs = String::new();
    let mut cargotoml =
        cargo_toml::Manifest::from_path("../raw-vulkan-library/Cargo.toml").unwrap();
    cargotoml.features.clear();
    lib_rs += "#![cfg_attr(not(test), no_std)]\n";
    for p @ Platform { platform, cfg_pred } in platforms {
        let code = generate_platform(platform);
        let mod_name = p.mod_name();
        fs::write(
            format!("../raw-vulkan-library/src/vulkan_core_{mod_name}.rs"),
            code,
        )
        .unwrap();
        let feature_flag = format!("force_enable_{platform}");
        lib_rs += &format!("#[cfg(any({cfg_pred}, feature = \"{feature_flag}\"))]\n");
        lib_rs += &format!("mod vulkan_core_{mod_name};\n");
        lib_rs += &format!("#[cfg(any({cfg_pred}, feature = \"{feature_flag}\"))]\n");
        lib_rs += &format!("pub use vulkan_core_{mod_name}::*;\n");
        cargotoml.features.insert(feature_flag, Vec::new());
    }
    fs::write("../raw-vulkan-library/src/lib.rs", lib_rs).unwrap();
    let new_cargo_toml = toml::to_string_pretty(&cargotoml).unwrap();
    fs::write("../raw-vulkan-library/Cargo.toml", new_cargo_toml).unwrap();
}

struct Platform {
    platform: &'static str,
    cfg_pred: &'static str,
}

impl Platform {
    fn mod_name(&self) -> String {
        self.platform.replace("-", "_")
    }
}
