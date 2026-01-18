#!/usr/bin/env bash
# Write the latest moteOS UEFI ISO to a removable USB device.

set -euo pipefail

usage() {
    cat <<EOF
Usage: $(basename "$0") /dev/diskN

This script overwrites the supplied removable disk with the current
UEFI ISO. It will unmount the disk, dd the ISO to the raw device,
and sync before ejecting.

You must run it as root.
EOF
}

if [[ $# -ne 1 ]]; then
    usage
    exit 1
fi

if [[ "$(id -u)" -ne 0 ]]; then
    echo "Error: This script must be run as root." >&2
    exit 1
fi

ISO_PATH="$(pwd)/moteos-x64-uefi.iso"
if [[ ! -f "$ISO_PATH" ]]; then
    echo "Error: ISO not found at $ISO_PATH" >&2
    exit 1
fi

DEV="$1"

if [[ "$(uname)" == "Darwin" ]]; then
    RAW_DEV="/dev/r${DEV##*/}"
else
    RAW_DEV="$DEV"
fi

echo "Writing $ISO_PATH to $DEV (raw $RAW_DEV)..."

# Unmount existing partitions
if [[ "$(uname)" == "Darwin" ]]; then
    diskutil unmountDisk "$DEV" >/dev/null || true
else
    for part in $(ls ${DEV}?* 2>/dev/null || true); do
        umount "$part" 2>/dev/null || true
    done
fi

dd if="$ISO_PATH" of="$RAW_DEV" bs=4m status=progress conv=sync
sync

echo "Ejecting $DEV..."
if [[ "$(uname)" == "Darwin" ]]; then
    diskutil eject "$DEV" >/dev/null || true
else
    eject "$DEV" >/dev/null || true
fi

echo "Done. USB should now be bootable."
