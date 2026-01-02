{
  description = "SZR35 Miryoku Keyboard - Complete Setup";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        # Python environment for overlay and trainer
        pythonEnv = pkgs.python3.withPackages (ps: with ps; [
          pyqt6
          hidapi
          evdev
          rich
        ]);

        # Build script as a Nix package
        buildFirmware = pkgs.writeShellScriptBin "build-firmware" ''
          set -e

          REPO_DIR="''${1:-$PWD}"
          QMK_CACHE="$HOME/.cache/szr35-vial-qmk"

          echo "Building SZR35 Miryoku firmware..."

          # Check Docker
          if ! command -v docker &> /dev/null; then
            echo "Error: Docker is not installed"
            exit 1
          fi

          # Clone vial-qmk if needed (cached in home dir)
          if [ ! -d "$QMK_CACHE" ]; then
            echo "Cloning vial-qmk (first time, ~500MB)..."
            ${pkgs.git}/bin/git clone --depth 1 https://github.com/vial-kb/vial-qmk.git "$QMK_CACHE"
            cd "$QMK_CACHE" && make git-submodule
          fi

          # Sync keyboard files
          echo "Syncing keyboard definition..."
          rm -rf "$QMK_CACHE/keyboards/szrkbd"
          cp -r "$REPO_DIR/qmk/szrkbd" "$QMK_CACHE/keyboards/"

          # Build
          echo "Running QMK build..."
          docker run --rm \
            -v "$QMK_CACHE:/qmk_firmware" \
            -w /qmk_firmware \
            qmkfm/qmk_cli:latest \
            make szrkbd/szr35:vial

          # Copy output
          mkdir -p "$REPO_DIR/firmware"
          if [ -f "$QMK_CACHE/szrkbd_szr35_vial.bin" ]; then
            cp "$QMK_CACHE/szrkbd_szr35_vial.bin" "$REPO_DIR/firmware/"
          elif [ -f "$QMK_CACHE/.build/szrkbd_szr35_vial.bin" ]; then
            cp "$QMK_CACHE/.build/szrkbd_szr35_vial.bin" "$REPO_DIR/firmware/"
          fi

          echo ""
          echo "Success! Firmware: firmware/szrkbd_szr35_vial.bin"
        '';

        # Flash script with recovery info
        flashFirmware = pkgs.writeShellScriptBin "flash-firmware" ''
          FIRMWARE="''${1:-firmware/szrkbd_szr35_vial.bin}"

          echo "=== SZR35 Firmware Flash ==="
          echo ""
          echo "IMPORTANT: Each half must be flashed separately!"
          echo ""
          echo "To enter DFU mode:"
          echo "  1. Locate boot pads: white square with 2 dots near thumb cluster"
          echo "     (NOT the one opposite USB port, the one slightly to the side)"
          echo "  2. Short the pads with tweezers/paperclip"
          echo "  3. While shorted, plug in USB"
          echo "  4. Release after connected"
          echo ""
          echo "If keyboard is bricked (no DFU detected):"
          echo "  - Try different USB cables (data cable, not charge-only)"
          echo "  - Hold boot pads for full 5 seconds while connecting"
          echo "  - Check: lsusb | grep -i stm"
          echo ""

          # Check for DFU device
          if ! lsusb | grep -qi "0483:df11\|STM.*DFU"; then
            echo "No DFU device detected!"
            echo "Put keyboard in DFU mode and try again."
            exit 1
          fi

          echo "DFU device found! Flashing..."
          sudo ${pkgs.dfu-util}/bin/dfu-util -a 0 -s 0x08000000:leave -D "$FIRMWARE"

          echo ""
          echo "Done! Keyboard should restart automatically."
          echo "If not working, unplug and replug the USB cable."
        '';

        # HID permissions helper
        fixHidPerms = pkgs.writeShellScriptBin "fix-hid-perms" ''
          echo "Setting HID permissions for SZR35..."
          sudo chmod 666 /dev/hidraw*
          echo "Done! Overlay/trainer should now work."
          echo ""
          echo "For permanent fix, add udev rule:"
          echo '  echo '\''SUBSYSTEM=="hidraw", ATTRS{idVendor}=="3601", ATTRS{idProduct}=="45d4", MODE="0666"'\'' | sudo tee /etc/udev/rules.d/99-szr35.rules'
          echo "  sudo udevadm control --reload-rules && sudo udevadm trigger"
        '';

      in {
        # Development shell
        devShells.default = pkgs.mkShell {
          buildInputs = [
            pythonEnv
            pkgs.dfu-util
            pkgs.docker
            pkgs.usbutils
            pkgs.git
            buildFirmware
            flashFirmware
            fixHidPerms
          ];

          shellHook = ''
            echo ""
            echo "╔═══════════════════════════════════════════════╗"
            echo "║     SZR35 Miryoku Development Environment     ║"
            echo "╚═══════════════════════════════════════════════╝"
            echo ""
            echo "Commands:"
            echo "  trainer-hid    - Terminal trainer (auto layer detection)"
            echo "  trainer        - Terminal trainer (manual mode)"
            echo "  overlay        - GUI layer overlay"
            echo "  build          - Build firmware (Docker)"
            echo "  flash          - Flash firmware to keyboard"
            echo "  fix-hid        - Fix HID permissions for overlay"
            echo ""
            echo "Keyboard must be in DFU mode to flash!"
            echo ""

            # Aliases for convenience
            alias trainer="python $PWD/overlay/miryoku_trainer.py"
            alias trainer-hid="python $PWD/overlay/miryoku_trainer.py --hid"
            alias overlay="python $PWD/overlay/miryoku_overlay.py"
            alias build="build-firmware $PWD"
            alias flash="flash-firmware $PWD/firmware/szrkbd_szr35_vial.bin"
            alias fix-hid="fix-hid-perms"
          '';
        };

        # Apps for running outside shell
        apps = {
          overlay = {
            type = "app";
            program = toString (pkgs.writeShellScript "overlay" ''
              ${pythonEnv}/bin/python ${self}/overlay/miryoku_overlay.py "$@"
            '');
          };

          trainer = {
            type = "app";
            program = toString (pkgs.writeShellScript "trainer" ''
              ${pythonEnv}/bin/python ${self}/overlay/miryoku_trainer.py "$@"
            '');
          };

          build = {
            type = "app";
            program = "${buildFirmware}/bin/build-firmware";
          };

          flash = {
            type = "app";
            program = "${flashFirmware}/bin/flash-firmware";
          };

          default = self.apps.${system}.trainer;
        };

        # Packages
        packages = {
          build-firmware = buildFirmware;
          flash-firmware = flashFirmware;
          fix-hid-perms = fixHidPerms;
        };
      }
    );
}
