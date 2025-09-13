PACKAGE_NAME = ui-act
VERSION = 0.1.0
ARCH = amd64
DEB_PACKAGE = $(PACKAGE_NAME)_$(VERSION)_$(ARCH).deb

BUILD_DIR = build
INSTALL_DIR = $(BUILD_DIR)/install
DEBIAN_DIR = $(BUILD_DIR)/debian

RUST_TARGET = release
RUST_BINARY = ui_act/target/$(RUST_TARGET)/ui-act

GNOME_EXT_UUID = ui-act@tobiasnorlund.github.com
GNOME_EXT_DIR = $(INSTALL_DIR)/usr/share/gnome-shell/extensions/$(GNOME_EXT_UUID)
BIN_DIR = $(INSTALL_DIR)/usr/bin

.PHONY: all clean build package install-deps build-rust build-extension prepare-install create-deb

all: package

install-deps:
	@echo "Installing build dependencies..."
	sudo apt-get update
	sudo apt-get install -y build-essential debhelper devscripts fakeroot

build-rust:
	@echo "Building Rust binary..."
	cd ui_act && cargo build --release

build-extension:
	@echo "Preparing GNOME extension..."
	cd launcher/schemas && glib-compile-schemas .

build: build-rust build-extension

prepare-install: build
	@echo "Preparing installation directory..."
	mkdir -p $(GNOME_EXT_DIR)
	mkdir -p $(BIN_DIR)
	mkdir -p $(DEBIAN_DIR)
	
	@echo "Copying GNOME extension files..."
	cp -r launcher/* $(GNOME_EXT_DIR)/
	
	@echo "Copying Rust binary..."
	cp $(RUST_BINARY) $(BIN_DIR)/ui-act

package: prepare-install
	@echo "Creating debian control files..."
	
	@# Create DEBIAN directory
	mkdir -p $(INSTALL_DIR)/DEBIAN
	
	@# Create control file
	@echo "Package: $(PACKAGE_NAME)" > $(INSTALL_DIR)/DEBIAN/control
	@echo "Version: $(VERSION)" >> $(INSTALL_DIR)/DEBIAN/control
	@echo "Architecture: $(ARCH)" >> $(INSTALL_DIR)/DEBIAN/control
	@echo "Maintainer: Tobias Norlund <tobias@norlund.se>" >> $(INSTALL_DIR)/DEBIAN/control
	@echo "Description: UI Act - Linux desktop agent" >> $(INSTALL_DIR)/DEBIAN/control
	@echo " UI Act is a Linux desktop agent that can perform tasks" >> $(INSTALL_DIR)/DEBIAN/control
	@echo " by interacting with your desktop GUI using Multi-Pointer X." >> $(INSTALL_DIR)/DEBIAN/control
	@echo "Homepage: https://github.com/tobiasnorlund/CoX" >> $(INSTALL_DIR)/DEBIAN/control
	@echo "Section: utils" >> $(INSTALL_DIR)/DEBIAN/control
	@echo "Priority: optional" >> $(INSTALL_DIR)/DEBIAN/control
	@echo "Depends: gnome-shell" >> $(INSTALL_DIR)/DEBIAN/control
	
	@# Create postinst script for extension installation
	@echo "#!/bin/bash" > $(INSTALL_DIR)/DEBIAN/postinst
	@echo "set -e" >> $(INSTALL_DIR)/DEBIAN/postinst
	@echo "if [ \"\$$1\" = \"configure\" ]; then" >> $(INSTALL_DIR)/DEBIAN/postinst
	@echo "    echo 'UI Act installed successfully.'" >> $(INSTALL_DIR)/DEBIAN/postinst
	@echo "    echo 'Enable the GNOME extension with: gnome-extensions enable $(GNOME_EXT_UUID)'" >> $(INSTALL_DIR)/DEBIAN/postinst
	@echo "" >> $(INSTALL_DIR)/DEBIAN/postinst
	@echo "    # Configure udev rules for uinput access" >> $(INSTALL_DIR)/DEBIAN/postinst
	@echo "    UDEV_RULE='KERNEL==\"uinput\", MODE=\"0660\", GROUP=\"input\"'" >> $(INSTALL_DIR)/DEBIAN/postinst
	@echo "    UDEV_FILE=/etc/udev/rules.d/99-uinput.rules" >> $(INSTALL_DIR)/DEBIAN/postinst
	@echo "    if ! grep -q \"\$$UDEV_RULE\" \"\$$UDEV_FILE\" 2>/dev/null; then" >> $(INSTALL_DIR)/DEBIAN/postinst
	@echo "        echo \"Adding udev rule for uinput access...\"" >> $(INSTALL_DIR)/DEBIAN/postinst
	@echo "        echo \"\$$UDEV_RULE\" >> \"\$$UDEV_FILE\"" >> $(INSTALL_DIR)/DEBIAN/postinst
	@echo "        udevadm control --reload-rules" >> $(INSTALL_DIR)/DEBIAN/postinst
	@echo "        udevadm trigger" >> $(INSTALL_DIR)/DEBIAN/postinst
	@echo "    fi" >> $(INSTALL_DIR)/DEBIAN/postinst
	@echo "fi" >> $(INSTALL_DIR)/DEBIAN/postinst
	chmod 755 $(INSTALL_DIR)/DEBIAN/postinst
	
	@# Create prerm script
	@echo "#!/bin/bash" > $(INSTALL_DIR)/DEBIAN/prerm
	@echo "set -e" >> $(INSTALL_DIR)/DEBIAN/prerm
	@echo "if [ \"\$$1\" = \"remove\" ]; then" >> $(INSTALL_DIR)/DEBIAN/prerm
	@echo "    gnome-extensions disable $(GNOME_EXT_UUID) 2>/dev/null || true" >> $(INSTALL_DIR)/DEBIAN/prerm
	@echo "fi" >> $(INSTALL_DIR)/DEBIAN/prerm
	chmod 755 $(INSTALL_DIR)/DEBIAN/prerm
	
	@echo "Building deb package..."
	dpkg-deb --build --root-owner-group $(INSTALL_DIR) $(DEB_PACKAGE)
	@echo "Package created: $(DEB_PACKAGE)"

install: package
	@echo "Installing deb package..."
	sudo dpkg -i $(DEB_PACKAGE)
	sudo apt-get install -f  # Fix any dependency issues

uninstall:
	@echo "Uninstalling $(PACKAGE_NAME)..."
	sudo dpkg -r $(PACKAGE_NAME)

clean:
	@echo "Cleaning build artifacts..."
	rm -rf $(BUILD_DIR)
	rm -f *.deb
	cd ui_act && cargo clean

test:
	@echo "Running tests..."
	cd ui_act && cargo test

help:
	@echo "Available targets:"
	@echo "  all        - Build and package (default)"
	@echo "  build      - Build Rust binary and prepare extension"
	@echo "  package    - Create deb package"
	@echo "  install    - Install the deb package"
	@echo "  uninstall  - Remove the package"
	@echo "  clean      - Clean build artifacts"
	@echo "  test       - Run tests"
	@echo "  help       - Show this help"
