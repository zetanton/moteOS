#!/usr/bin/env bash
# Create a FAT32 UEFI stick containing the latest ISO payload

set -euo pipefail

usage() {
  cat <<EOF
Usage: $(basename "$0") /dev/diskN

Formats the given disk as GPT+FAT32, mounts it, and installs
the EFI application from the latest ISO into EFI/BOOT/BOOTX64.EFI.

Run as root. Works on macOS (diskutil + hdiutil) and Linux (parted + mkfs.vfat).
EOF
}

if [[ $# -ne 1 ]]; then
  usage
  exit 1
fi
if [[ "$(id -u)" -ne 0 ]]; then
  echo "Must run as root" >&2
  exit 1
fi

ISO="$(pwd)/moteos-x64-uefi.iso"
ISO_MOUNT="$(mktemp -d)"
cleanup_iso() {
    rm -rf "$ISO_MOUNT"
}
trap cleanup_iso EXIT

command -v xorriso >/dev/null 2>&1 || { echo "xorriso required for ISO extraction" >&2; exit 1; }
if [[ ! -f "$ISO" ]]; then
  echo "ISO not found at $ISO" >&2
  exit 2
fi

DISK="$1"
DEV_HOST=""
if [[ "$(uname)" == "Darwin" ]]; then
  PARTED="diskutil"
  ROOT_DISK="$DISK"
  diskutil unmountDisk "$DISK" >/dev/null || true
  diskutil partitionDisk "$DISK" GPT FAT32 MOTEOS 100% >/dev/null
  BASE="${DISK##*/}"
  NUM="${BASE#disk}"
  EFI_VOL=$(diskutil mount disk"${NUM}"s1 | awk -F': ' '/mount/ {print $NF}')
  if [[ -z "$EFI_VOL" ]]; then
      echo "Failed to mount EFI partition" >&2
      exit 3
  fi
  TARGET="$EFI_VOL"
else
  PARTED="parted"
  RAW="$DISK"
  echo "Creating GPT on $DISK..."
  parted "$DISK" --script mklabel gpt
  parted "$DISK" --script mkpart primary fat32 1MiB 100%
  parted "$DISK" --script name 1 EFI
  parted "$DISK" --script set 1 boot on
  mkfs.vfat -F32 "${DISK}1"
  TARGET="/mnt/moteos-iso"
  mkdir -p "$TARGET"
  mount "${DISK}1" "$TARGET"
fi

  xorriso -osirrox on -indev "$ISO" -extract / "$ISO_MOUNT" >/dev/null
  mkdir -p "$TARGET/EFI"
  cp -R "$ISO_MOUNT/EFI/BOOT" "$TARGET/EFI/"
  cp "$ISO_MOUNT/startup.nsh" "$TARGET/" || true
sync

if [[ "$(uname)" == "Darwin" ]]; then
  diskutil eject "$DISK"
else
  umount "$TARGET"
fi

echo "USB written with FAT32 EFI payload. Boot using 'UEFI: <USB>' entry."
