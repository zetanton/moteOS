// EFI variable storage implementation
// Stores configuration in EFI variables for persistence across reboots

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use uefi::{
    prelude::*,
    table::runtime::{VariableAttributes, VariableVendor},
    CString16,
};
use crate::error::ConfigError;
use crate::storage::ConfigStorage;
use crate::toml::{TomlParser, Value};

/// EFI variable name for moteOS configuration
const CONFIG_VARIABLE_NAME: &str = "MoteOS-Config";

/// Custom GUID for moteOS EFI variables
/// Generated UUID: 8a4e8e1e-3c5f-4a9b-9d2e-1f3a5b7c9d0e
const MOTEOS_VENDOR_GUID: uefi::Guid = uefi::Guid::from_fields(
    0x8a4e8e1e,
    0x3c5f,
    0x4a9b,
    0x9d,
    0x2e,
    &[0x1f, 0x3a, 0x5b, 0x7c, 0x9d, 0x0e],
);

/// EFI variable storage for configuration
/// 
/// This implementation stores configuration in EFI variables, which persist
/// across reboots and are accessible from both UEFI boot services and runtime.
/// 
/// The configuration is stored as a TOML string in the EFI variable "MoteOS-Config"
/// with a custom vendor GUID.
pub struct EfiConfigStorage {
    /// System table reference (only valid during boot services)
    /// After exit_boot_services, we use runtime services
    system_table: Option<&'static SystemTable<Runtime>>,
}

impl EfiConfigStorage {
    /// Create a new EFI config storage instance
    /// 
    /// # Arguments
    /// * `system_table` - UEFI system table (can be None after exit_boot_services)
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

        let vendor = VariableVendor::Custom(MOTEOS_VENDOR_GUID);

        // Try to get runtime services
        let rt = self
            .system_table
            .ok_or_else(|| ConfigError::efi_error("System table not available"))?
            .runtime_services();

        // Read variable with maximum size (64KB EFI variable limit)
        let mut buffer = [0u8; 65536];

        match rt.get_variable(name, vendor, &mut buffer) {
            Ok((size, _attrs)) => {
                let data = buffer[..size].to_vec();
                Ok(Some(data))
            }
            Err(uefi::Status::NOT_FOUND) => Ok(None),
            Err(status) => {
                let msg = format!("Failed to read EFI variable: {:?}", status);
                Err(ConfigError::efi_error(&msg))
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

        let vendor = VariableVendor::Custom(MOTEOS_VENDOR_GUID);

        // Attributes: non-volatile, boot service access, runtime access
        let attributes = VariableAttributes::NON_VOLATILE
            | VariableAttributes::BOOTSERVICE_ACCESS
            | VariableAttributes::RUNTIME_ACCESS;

        let rt = self
            .system_table
            .ok_or_else(|| ConfigError::efi_error("System table not available"))?
            .runtime_services();

        rt.set_variable(name, vendor, attributes, data)
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
                let toml_str = core::str::from_utf8(&data)
                    .map_err(|e| {
                        ConfigError::deserialization_error(&format!(
                            "Invalid UTF-8 in EFI variable: {}",
                            e
                        ))
                    })?;

                // Parse TOML
                let value = TomlParser::parse(toml_str)
                    .map_err(|e| {
                        ConfigError::deserialization_error(&format!(
                            "Failed to parse TOML: {:?}",
                            e
                        ))
                    })?;

                Ok(Some(value))
            }
        }
    }

    fn save(&mut self, config: &Value) -> Result<(), ConfigError> {
        // Serialize to TOML
        let toml_str = TomlParser::serialize(config)
            .map_err(|e| {
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
