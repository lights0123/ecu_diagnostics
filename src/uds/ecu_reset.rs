//!  Provides methods to reset the ECU in order to simulate power cycling and resetting memory regions

use super::{lookup_uds_nrc, UDSCommand, UdsDiagnosticServer};
use crate::{DiagError, DiagServerResult, DiagnosticServer};

/// Options for resetting the ECU
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResetType {
    /// Signals the ECU to perform a hard-reset,
    /// simulating a forceful power off/on cycle
    ///
    /// This might result in both non-volatile memory and volatile memory locations being re-initialized
    HardReset,

    /// Signals the ECU to perform a simulated key off/on cycle,
    /// simulating the usual key-off/on cycle
    ///
    /// This typically results in the preservation of non-volatile memory,
    /// but volatile memory will be re-initialized
    KeyOffReset,

    /// Signals the ECU to perform a soft reset, simply rebooting the current
    /// application running on it.
    ///
    /// This will result in the preservation of both non-volatile and volatile memory
    SoftReset,

    /// Enables a rapid power shutdown on the ECU during a key-off cycle.
    ///
    /// IMPORTANT: Once this has been used, the diagnostic server **cannot** send
    /// any other messages other than ECUReset in order to not disturb the rapid power
    /// shutdown function.
    EnableRapidPowerShutDown,

    /// Disables a rapid power shutdown on the ECU during a key-off cycle.
    DisableRapidPowerShutDown,

    /// Other OEM defined power mode
    Other(u8),
}

impl From<ResetType> for u8 {
    fn from(from: ResetType) -> Self {
        match from {
            ResetType::HardReset => 0x01,
            ResetType::KeyOffReset => 0x02,
            ResetType::SoftReset => 0x03,
            ResetType::EnableRapidPowerShutDown => 0x04,
            ResetType::DisableRapidPowerShutDown => 0x05,
            ResetType::Other(x) => x,
        }
    }
}

impl UdsDiagnosticServer {
    /// Asks the ECU to perform a hard reset. See [ResetType::HardReset] for more details
    ///
    /// ## Parameters
    /// * server - The UDS Diagnostic server
    pub fn ecu_hard_reset(&mut self) -> DiagServerResult<()> {
        self
            .execute_command_with_response(UDSCommand::ECUReset, &[ResetType::HardReset.into()])
            .map(|_| ())
    }

    /// Asks the ECU to perform a key off/on reset. See [ResetType::KeyOffReset] for more details
    ///
    /// ## Parameters
    /// * server - The UDS Diagnostic server
    pub fn ecu_key_off_on_reset(&mut self) -> DiagServerResult<()> {
        self
            .execute_command_with_response(UDSCommand::ECUReset, &[ResetType::KeyOffReset.into()])
            .map(|_| ())
    }

    /// Asks the ECU to perform a soft reset. See [ResetType::SoftReset] for more details
    ///
    /// ## Parameters
    /// * server - The UDS Diagnostic server
    pub fn ecu_soft_reset(&mut self) -> DiagServerResult<()> {
        self
            .execute_command_with_response(UDSCommand::ECUReset, &[ResetType::SoftReset.into()])
            .map(|_| ())
    }

    /// Asks the ECU to enable rapid power shutdown mode. See [ResetType::EnableRapidPowerShutDown] for more details
    ///
    /// ## Parameters
    /// * server - The UDS Diagnostic server
    ///
    /// ## Returns
    /// If successful, this function will return the minimum time in seconds that the ECU will remain in the power-down sequence
    pub fn enable_rapid_power_shutdown(&mut self) -> DiagServerResult<u8> {
        let res = self.execute_command_with_response(
            UDSCommand::ECUReset,
            &[ResetType::EnableRapidPowerShutDown.into()],
        )?;
        match res.get(2) {
            Some(&time) if time == 0xFF => Err(DiagError::ECUError {
                code: 0x10,
                def: Some(lookup_uds_nrc(0x10)),
            }), // General reject
            Some(&time) => Ok(time),
            None => Err(DiagError::InvalidResponseLength),
        }
    }

    /// Asks the ECU to disable rapid power shutdown mode
    ///
    /// ## Parameters
    /// * server - The UDS Diagnostic server
    pub fn disable_rapid_power_shutdown(&mut self) -> DiagServerResult<()> {
        self
            .execute_command_with_response(
                UDSCommand::ECUReset,
                &[ResetType::DisableRapidPowerShutDown.into()],
            )
            .map(|_| ())
    }
}
