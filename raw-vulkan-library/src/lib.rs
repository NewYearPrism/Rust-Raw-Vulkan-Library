#![cfg_attr(not(test), no_std)]
#[cfg(any(all(target_arch = "x86_64", unix), feature = "force_enable_x86_64-unknown-linux-gnu"))]
mod vulkan_core_x86_64_unknown_linux_gnu;
#[cfg(any(all(target_arch = "x86_64", unix), feature = "force_enable_x86_64-unknown-linux-gnu"))]
pub use vulkan_core_x86_64_unknown_linux_gnu::*;
#[cfg(any(all(target_arch = "x86_64", windows), feature = "force_enable_x86_64-pc-windows-msvc"))]
mod vulkan_core_x86_64_pc_windows_msvc;
#[cfg(any(all(target_arch = "x86_64", windows), feature = "force_enable_x86_64-pc-windows-msvc"))]
pub use vulkan_core_x86_64_pc_windows_msvc::*;
#[cfg(any(all(target_arch = "x86", windows), feature = "force_enable_i686-pc-windows-msvc"))]
mod vulkan_core_i686_pc_windows_msvc;
#[cfg(any(all(target_arch = "x86", windows), feature = "force_enable_i686-pc-windows-msvc"))]
pub use vulkan_core_i686_pc_windows_msvc::*;
#[cfg(any(all(target_arch = "x86", unix), feature = "force_enable_i686-unknown-linux-gnu"))]
mod vulkan_core_i686_unknown_linux_gnu;
#[cfg(any(all(target_arch = "x86", unix), feature = "force_enable_i686-unknown-linux-gnu"))]
pub use vulkan_core_i686_unknown_linux_gnu::*;
#[cfg(any(all(target_arch = "aarch64", unix), feature = "force_enable_aarch64-unknown-linux-gnu"))]
mod vulkan_core_aarch64_unknown_linux_gnu;
#[cfg(any(all(target_arch = "aarch64", unix), feature = "force_enable_aarch64-unknown-linux-gnu"))]
pub use vulkan_core_aarch64_unknown_linux_gnu::*;
#[cfg(any(all(target_arch = "aarch64", target_os = "android"), feature = "force_enable_aarch64-linux-android"))]
mod vulkan_core_aarch64_linux_android;
#[cfg(any(all(target_arch = "aarch64", target_os = "android"), feature = "force_enable_aarch64-linux-android"))]
pub use vulkan_core_aarch64_linux_android::*;
