# Makefile for moteOS ISO generation and QEMU testing
# See docs/TECHNICAL_SPECIFICATIONS.md Section 3.8.8-9

.PHONY: help iso-uefi iso-bios iso-all test-boot test-network test-api test-all clean

# Default target
help:
	@echo "moteOS Build and Test Targets"
	@echo ""
	@echo "ISO Generation:"
	@echo "  make iso-uefi      - Build UEFI boot ISO (x86_64)"
	@echo "  make iso-bios       - Build BIOS boot ISO (x86_64)"
	@echo "  make iso-all        - Build both UEFI and BIOS ISOs"
	@echo ""
	@echo "QEMU Testing:"
	@echo "  make test-boot      - Test kernel boot in QEMU"
	@echo "  make test-network   - Test network connectivity in QEMU"
	@echo "  make test-api       - Test LLM API connectivity in QEMU"
	@echo "  make test-all       - Run all tests"
	@echo ""
	@echo "Utilities:"
	@echo "  make clean         - Clean build artifacts and ISOs"
	@echo "  make help          - Show this help message"

# ISO Generation Targets
iso-uefi:
	@echo "Building UEFI boot ISO..."
	@chmod +x tools/build-iso-uefi.sh
	@./tools/build-iso-uefi.sh

iso-bios:
	@echo "Building BIOS boot ISO..."
	@chmod +x tools/build-iso-bios.sh
	@./tools/build-iso-bios.sh

iso-all: iso-uefi iso-bios
	@echo "All ISOs built successfully"

# QEMU Test Targets
test-boot:
	@echo "Running boot test..."
	@chmod +x tools/test-boot.sh
	@./tools/test-boot.sh

test-network:
	@echo "Running network test..."
	@chmod +x tools/test-network.sh
	@./tools/test-network.sh

test-api:
	@echo "Running API test..."
	@chmod +x tools/test-api.sh
	@./tools/test-api.sh

test-all: test-boot test-network test-api
	@echo "All tests completed"

# Clean target
clean:
	@echo "Cleaning build artifacts..."
	@rm -rf iso iso-bios
	@rm -f moteos-x64-uefi.iso moteos-x64-bios.iso
	@rm -f /tmp/moteos-*-test.log
	@rm -f /tmp/mock-api-server.py /tmp/moteos-test-response.txt
	@echo "Clean complete"

# Convenience targets for quick testing
run-qemu-uefi: iso-uefi
	@echo "Starting QEMU with UEFI ISO..."
	@chmod +x tools/test-boot.sh
	@./tools/test-boot.sh

run-qemu-bios: iso-bios
	@echo "Starting QEMU with BIOS ISO..."
	@echo "Note: BIOS boot requires proper Multiboot2 setup"
	@qemu-system-x86_64 \
		-machine q35 \
		-cpu qemu64 \
		-m 1G \
		-cdrom moteos-x64-bios.iso \
		-netdev user,id=net0 \
		-device virtio-net,netdev=net0 \
		-serial stdio \
		-display none \
		-no-reboot
