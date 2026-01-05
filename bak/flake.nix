{
  description = "SZR35 Keyboard - Complete Setup";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        # Python environment for legacy overlay and trainer
        pythonEnv = pkgs.python3.withPackages (ps: with ps; [
          pyqt6
          hidapi
          evdev
          rich
        ]);

        # Rust trainer package
        szr35Trainer = pkgs.rustPlatform.buildRustPackage {
          pname = "szr35-trainer";
          version = "0.1.0";
          src = ./trainer;
          cargoLock.lockFile = ./trainer/Cargo.lock;

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

          postInstall = ''
            mkdir -p $out/share/applications
            cp ${./trainer/szr35-trainer.desktop} $out/share/applications/
            substituteInPlace $out/share/applications/szr35-trainer.desktop \
              --replace "Exec=szr35-trainer" "Exec=$out/bin/szr35-trainer"
          '';

          meta = with pkgs.lib; {
            description = "SZR35 keyboard layout trainer";
            license = licenses.mit;
          };
        };

        # Build firmware script
        buildFirmware = pkgs.writeShellScriptBin "build-firmware" ''
          set -e

          REPO_DIR="''${1:-$PWD}"
          QMK_CACHE="$HOME/.cache/szr35-vial-qmk"
          KEYMAP="''${2:-miryoku}"

          echo "Building SZR35 firmware (keymap: $KEYMAP)..."

          if ! command -v docker &> /dev/null; then
            echo "Error: Docker is not installed"
            exit 1
          fi

          if [ ! -d "$QMK_CACHE" ]; then
            echo "Cloning vial-qmk (first time, ~500MB)..."
            ${pkgs.git}/bin/git clone --depth 1 https://github.com/vial-kb/vial-qmk.git "$QMK_CACHE"
            cd "$QMK_CACHE" && make git-submodule
          fi

          echo "Syncing keyboard definition..."
          rm -rf "$QMK_CACHE/keyboards/kbd"
          cp -r "$REPO_DIR/qmk/kbd" "$QMK_CACHE/keyboards/"

          echo "Running QMK build..."
          docker run --rm \
            -v "$QMK_CACHE:/qmk_firmware" \
            -w /qmk_firmware \
            qmkfm/qmk_cli:latest \
            make kbd/szr35:$KEYMAP

          mkdir -p "$REPO_DIR/firmware"
          if [ -f "$QMK_CACHE/kbd_szr35_''${KEYMAP}.bin" ]; then
            cp "$QMK_CACHE/kbd_szr35_''${KEYMAP}.bin" "$REPO_DIR/firmware/"
          elif [ -f "$QMK_CACHE/.build/kbd_szr35_''${KEYMAP}.bin" ]; then
            cp "$QMK_CACHE/.build/kbd_szr35_''${KEYMAP}.bin" "$REPO_DIR/firmware/"
          fi

          echo ""
          echo "Success! Firmware: firmware/kbd_szr35_''${KEYMAP}.bin"
        '';

        # Flash firmware script
        flashFirmware = pkgs.writeShellScriptBin "flash-firmware" ''
          FIRMWARE="''${1:-firmware/kbd_szr35_miryoku.bin}"

          echo "=== SZR35 Firmware Flash ==="
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

        # HID permissions helper
        fixHidPerms = pkgs.writeShellScriptBin "fix-hid-perms" ''
          echo "Setting HID permissions for SZR35..."
          sudo chmod 666 /dev/hidraw*
          echo "Done!"
        '';

      in {
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
            pkgs.rustc
            pkgs.cargo
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

          shellHook = ''
            echo ""
            echo "╔═══════════════════════════════════════════════╗"
            echo "║       SZR35 Keyboard Development Environment   ║"
            echo "╚═══════════════════════════════════════════════╝"
            echo ""
            echo "Commands:"
            echo "  trainer           - GUI overlay (runs in background)"
            echo "  trainer --tui     - Terminal trainer"
            echo "  trainer -v        - Verbose mode"
            echo "  build [keymap]    - Build firmware (default: miryoku)"
            echo "  flash [file]      - Flash firmware"
            echo "  fix-hid           - Fix HID permissions"
            echo ""
            echo "Keymaps: default, factory, miryoku"
            echo ""

            alias trainer="cargo run --manifest-path $PWD/trainer/Cargo.toml --release --"

            # Legacy Python
            alias py-trainer="python $PWD/overlay/miryoku_trainer.py"
            alias py-overlay="python $PWD/overlay/miryoku_overlay.py"

            alias build="build-firmware $PWD"
            alias flash="flash-firmware"
            alias fix-hid="fix-hid-perms"
          '';
        };

        apps = {
          trainer = {
            type = "app";
            program = "${szr35Trainer}/bin/szr35-trainer";
          };

          default = self.apps.${system}.trainer;

          build = {
            type = "app";
            program = "${buildFirmware}/bin/build-firmware";
          };

          flash = {
            type = "app";
            program = "${flashFirmware}/bin/flash-firmware";
          };
        };

        packages = {
          default = szr35Trainer;
          szr35-trainer = szr35Trainer;
          build-firmware = buildFirmware;
          flash-firmware = flashFirmware;
          fix-hid-perms = fixHidPerms;
        };
      }
    );
}
