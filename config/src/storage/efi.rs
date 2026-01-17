#![no_std]

// EFI variable storage implementation
// Stores configuration in EFI variables for persistence across reboots

extern crate alloc;

use crate::error::ConfigError;
use crate::storage::ConfigStorage;
use crate::toml::{TomlParser, Value};
use alloc::vec::Vec;

// UEFI types are only available when building for UEFI targets
// This code will not compile in host tests, which is expected
#[cfg(any(
    target_os = "uefi",
    all(target_arch = "x86_64", feature = "uefi"),
    all(target_arch = "aarch64", feature = "uefi")
))]
mod efi_impl {
    use super::*;
    use alloc::format;
    use uefi::{
        prelude::*,
        table::runtime::{VariableAttributes, VariableVendor},
        table::Runtime,
        CString16,
    };

    /// EFI variable name for moteOS configuration
    const CONFIG_VARIABLE_NAME: &str = "MoteOS-Config";

    /// Custom GUID for moteOS EFI variables
    /// Generated UUID: 8a4e8e1e-3c5f-4a9b-9d2e-1f3a5b7c9d0e
    const MOTEOS_VENDOR_GUID: uefi::Guid = uefi::Guid::new(
        [0x8a, 0x4e, 0x8e, 0x1e],
        [0x3c, 0x5f],
        [0x4a, 0x9b],
        0x9d,
        0x2e,
        [0x1f, 0x3a, 0x5b, 0x7c, 0x9d, 0x0e],
    );

    /// EFI variable storage for configuration
    ///
    /// This implementation stores configuration in EFI variables, which persist
    /// across reboots and are accessible from both UEFI boot services and runtime.
    ///
    /// The configuration is stored as a TOML string in the EFI variable "MoteOS-Config"
    /// with a custom vendor GUID.
    pub struct EfiConfigStorage {
        /// System table reference
        system_table: Option<&'static SystemTable<Runtime>>,
    }

    impl EfiConfigStorage {
        /// Create a new EFI config storage instance
        pub fn new(system_table: Option<&'static SystemTable<Runtime>>) -> Self {
            Self { system_table }
        }

        /// Get the variable name as a CString16
        fn variable_name(&self) -> Result<CString16, ConfigError> {
            CString16::try_from(CONFIG_VARIABLE_NAME)
                .map_err(|_| ConfigError::efi_error("Failed to create variable name"))
        }

        /// Read EFI variable
        fn read_variable(&self) -> Result<Option<Vec<u8>>, ConfigError> {
            let name = self.variable_name()?;
            let name = name.as_ref();

            // Note: VariableVendor is a newtype enum with predefined variants,
            // but we need a custom GUID. Use unsafe transmute as workaround.
            // This is safe because VariableVendor is repr(transparent) and only wraps a Guid.
            let vendor = unsafe {
                core::mem::transmute::<uefi::Guid, VariableVendor>(MOTEOS_VENDOR_GUID)
            };

            // Try to get runtime services
            let rt = unsafe {
                self.system_table
                    .ok_or_else(|| ConfigError::efi_error("System table not available"))?
                    .runtime_services()
            };

            // Read variable with maximum size (64KB EFI variable limit)
            let mut buffer = [0u8; 65536];

            match rt.get_variable(name, &vendor, &mut buffer) {
                Ok((data, _attrs)) => Ok(Some(data.to_vec())),
                Err(err) => {
                    if err.status() == uefi::Status::NOT_FOUND {
                        Ok(None)
                    } else {
                        let msg = format!("Failed to read EFI variable: {:?}", err.status());
                        Err(ConfigError::efi_error(&msg))
                    }
                }
            }
        }

        /// Write EFI variable
        fn write_variable(&self, data: &[u8]) -> Result<(), ConfigError> {
            // EFI variable size limit is 64KB
            if data.len() > 65536 {
                return Err(ConfigError::efi_error(
                    "Configuration too large for EFI variable (max 64KB)",
                ));
            }

            let name = self.variable_name()?;
            let name = name.as_ref();

            // Note: VariableVendor is a newtype enum with predefined variants,
            // but we need a custom GUID. Use unsafe transmute as workaround.
            let vendor = unsafe {
                core::mem::transmute::<uefi::Guid, VariableVendor>(MOTEOS_VENDOR_GUID)
            };

            // Attributes: non-volatile, boot service access, runtime access
            let attributes = VariableAttributes::NON_VOLATILE
                | VariableAttributes::BOOTSERVICE_ACCESS
                | VariableAttributes::RUNTIME_ACCESS;

            let rt = unsafe {
                self.system_table
                    .ok_or_else(|| ConfigError::efi_error("System table not available"))?
                    .runtime_services()
            };

            rt.set_variable(name, &vendor, attributes, data)
                .map_err(|status| {
                    let msg = format!("Failed to write EFI variable: {:?}", status);
                    ConfigError::efi_error(&msg)
                })
        }
    }

    impl ConfigStorage for EfiConfigStorage {
        fn load(&self) -> Result<Option<Value>, ConfigError> {
            match self.read_variable()? {
                None => Ok(None),
                Some(data) => {
                    // Convert bytes to string
                    let toml_str = core::str::from_utf8(&data).map_err(|e| {
                        ConfigError::deserialization_error(&format!(
                            "Invalid UTF-8 in EFI variable: {}",
                            e
                        ))
                    })?;

                    // Parse TOML
                    let value = TomlParser::parse(toml_str).map_err(|e| {
                        ConfigError::deserialization_error(&format!("Failed to parse TOML: {:?}", e))
                    })?;

                    Ok(Some(value))
                }
            }
        }

        fn save(&mut self, config: &Value) -> Result<(), ConfigError> {
            // Serialize to TOML
            let toml_str = TomlParser::serialize(config).map_err(|e| {
                ConfigError::serialization_error(&format!("Failed to serialize TOML: {:?}", e))
            })?;

            // Convert to bytes
            let data = toml_str.as_bytes();

            // Write to EFI variable
            self.write_variable(data)
        }

        fn exists(&self) -> bool {
            match self.read_variable() {
                Ok(Some(_)) => true,
                Ok(None) => false,
                Err(_) => false,
            }
        }
    }
}

// Re-export for UEFI targets only
#[cfg(any(
    target_os = "uefi",
    all(target_arch = "x86_64", feature = "uefi"),
    all(target_arch = "aarch64", feature = "uefi")
))]
pub use efi_impl::EfiConfigStorage;

// For non-UEFI targets (tests, etc.), provide a stub or alternative implementation
#[cfg(not(any(
    target_os = "uefi",
    all(target_arch = "x86_64", feature = "uefi"),
    all(target_arch = "aarch64", feature = "uefi")
)))]
pub struct EfiConfigStorage;

#[cfg(not(any(
    target_os = "uefi",
    all(target_arch = "x86_64", feature = "uefi"),
    all(target_arch = "aarch64", feature = "uefi")
)))]
impl EfiConfigStorage {
    pub fn new(_system_table: Option<()>) -> Self {
        Self
    }
}

#[cfg(not(any(
    target_os = "uefi",
    all(target_arch = "x86_64", feature = "uefi"),
    all(target_arch = "aarch64", feature = "uefi")
)))]
impl ConfigStorage for EfiConfigStorage {
    fn load(&self) -> Result<Option<Value>, ConfigError> {
        // Stub: return None for non-UEFI targets
        Ok(None)
    }

    fn save(&mut self, _config: &Value) -> Result<(), ConfigError> {
        // Stub: no-op for non-UEFI targets
        Err(ConfigError::efi_error("EFI storage not available on this platform"))
    }

    fn exists(&self) -> bool {
        false
    }
}
