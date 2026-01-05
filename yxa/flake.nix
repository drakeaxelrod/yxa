{
  description = "Yxa 36-Key Split Ergonomic Keyboard";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        # Rust toolchain with embedded target for firmware
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rustfmt" "clippy" ];
          targets = [ "thumbv7em-none-eabihf" ];
        };

        # Visual guide package
        yxaVisualGuide = pkgs.rustPlatform.buildRustPackage {
          pname = "yxa-visual-guide";
          version = "0.1.0";
          src = ./crates/visual-guide;
          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = with pkgs; [
            pkg-config
          ];

          buildInputs = with pkgs; [
            libxkbcommon
            libGL
            wayland
            xorg.libX11
            xorg.libXcursor
            xorg.libXrandr
            xorg.libXi
            vulkan-loader
          ];

          meta = with pkgs.lib; {
            description = "Yxa keyboard layout visual guide";
            license = licenses.mit;
          };
        };

        # Flash firmware script
        flashFirmware = pkgs.writeShellScriptBin "flash-yxa" ''
          FIRMWARE="''${1:-target/thumbv7em-none-eabihf/release/yxa-firmware.bin}"

          echo "=== Yxa Firmware Flash ==="
          echo ""
          echo "IMPORTANT: Each half must be flashed separately!"
          echo ""
          echo "To enter DFU mode:"
          echo "  1. Locate boot pads near thumb cluster"
          echo "  2. Short the pads with tweezers/paperclip"
          echo "  3. While shorted, plug in USB"
          echo "  4. Release after connected"
          echo ""

          if ! lsusb | grep -qi "0483:df11\|STM.*DFU"; then
            echo "No DFU device detected!"
            echo "Put keyboard in DFU mode and try again."
            exit 1
          fi

          echo "DFU device found! Flashing..."
          sudo ${pkgs.dfu-util}/bin/dfu-util -a 0 -s 0x08000000:leave -D "$FIRMWARE"

          echo ""
          echo "Done! Keyboard should restart automatically."
        '';

        # Build firmware and convert to binary
        buildFirmware = pkgs.writeShellScriptBin "build-firmware" ''
          set -e
          echo "Building Yxa firmware..."
          cargo build --release -p yxa-firmware

          echo "Converting to binary..."
          ${pkgs.gcc-arm-embedded}/bin/arm-none-eabi-objcopy \
            -O binary \
            target/thumbv7em-none-eabihf/release/yxa-firmware \
            target/thumbv7em-none-eabihf/release/yxa-firmware.bin

          echo ""
          echo "Success! Binary: target/thumbv7em-none-eabihf/release/yxa-firmware.bin"
          ls -lh target/thumbv7em-none-eabihf/release/yxa-firmware.bin
        '';

        # HID permissions helper
        fixHidPerms = pkgs.writeShellScriptBin "fix-hid-perms" ''
          echo "Setting HID permissions for Yxa..."
          sudo chmod 666 /dev/hidraw*
          echo "Done!"
        '';

      in {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            rustToolchain
            pkgs.dfu-util
            pkgs.usbutils
            pkgs.gcc-arm-embedded
            pkgs.probe-rs-tools
            pkgs.flip-link
            pkgs.pkg-config
            pkgs.libxkbcommon
            pkgs.libGL
            pkgs.wayland
            pkgs.xorg.libX11
            pkgs.xorg.libXcursor
            pkgs.xorg.libXrandr
            pkgs.xorg.libXi
            pkgs.vulkan-loader
            pkgs.mold
            pkgs.clang
            buildFirmware
            flashFirmware
            fixHidPerms
          ];

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
            pkgs.libxkbcommon
            pkgs.libGL
            pkgs.wayland
            pkgs.xorg.libX11
            pkgs.xorg.libXcursor
            pkgs.xorg.libXrandr
            pkgs.xorg.libXi
            pkgs.vulkan-loader
          ];

          CARGO_TARGET_THUMBV7EM_NONE_EABIHF_RUNNER = "probe-rs run --chip STM32F401CCUx";

          shellHook = ''
            echo ""
            echo "╔═══════════════════════════════════════════════╗"
            echo "║           Yxa Keyboard Development            ║"
            echo "╚═══════════════════════════════════════════════╝"
            echo ""
            echo "Crates:"
            echo "  yxa-firmware      - Keyberon firmware (STM32F401)"
            echo "  yxa-visual-guide  - Layout trainer/overlay GUI"
            echo ""
            echo "Commands:"
            echo "  build-firmware    - Build firmware and convert to .bin"
            echo "  flash-yxa         - Flash firmware via DFU"
            echo "  fix-hid-perms     - Fix HID permissions"
            echo "  guide             - Run visual guide"
            echo ""

            alias guide="cargo run --release -p yxa-visual-guide --"
          '';
        };

        apps = {
          visual-guide = {
            type = "app";
            program = "${yxaVisualGuide}/bin/yxa-visual-guide";
          };

          default = self.apps.${system}.visual-guide;
        };

        packages = {
          default = yxaVisualGuide;
          yxa-visual-guide = yxaVisualGuide;
          build-firmware = buildFirmware;
          flash-yxa = flashFirmware;
          fix-hid-perms = fixHidPerms;
        };
      }
    );
}
