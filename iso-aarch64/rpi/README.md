# moteOS for Raspberry Pi

## Requirements

- Raspberry Pi 4 or Raspberry Pi 400 (ARM64 support)
- UEFI firmware (e.g., from https://github.com/pftf/RPi4)
- MicroSD card (8GB minimum, 16GB recommended)
- USB keyboard (for input)

## Installation

1. Flash the ISO to a MicroSD card:
   ```bash
   sudo dd if=moteos-aarch64-uefi.iso of=/dev/sdX bs=4M status=progress
   ```

2. Copy UEFI firmware files to the boot partition:
   - Download RPi4_UEFI_Firmware_v1.XX.zip from the RPi4 UEFI project
   - Extract and copy `RPI_EFI.fd` to the boot partition as `EFI/BOOT/startup.nsh`

3. Insert the MicroSD card into your Raspberry Pi

4. Power on the Raspberry Pi

5. The system should boot into moteOS

## Troubleshooting

- If the system doesn't boot, ensure UEFI firmware is properly installed
- Check that the MicroSD card is properly formatted
- Verify that `BOOTAA64.EFI` is in `EFI/BOOT/` directory
- For serial console debugging, connect to UART pins (GPIO 14/15)

## Notes

- Raspberry Pi 3 is not supported (32-bit only)
- Raspberry Pi 4 Model B is the primary target
- USB networking may require additional driver support
