{
  description = "Yxa Keyboard - Firmware and Visual Guide";

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

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rustfmt" "clippy" ];
        };

        # Visual guide package (binary only)
        yxaVisualGuideBin = pkgs.rustPlatform.buildRustPackage {
          pname = "yxa-visual-guide";
          version = "0.1.0";
          src = ./visual-guide;
          cargoLock.lockFile = ./visual-guide/Cargo.lock;

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
            description = "Yxa keyboard layout visual guide and trainer";
            license = licenses.mit;
          };
        };

        # Desktop entry for the visual guide
        desktopItem = pkgs.makeDesktopItem {
          name = "yxa-visual-guide";
          desktopName = "Yxa Visual Guide";
          comment = "Yxa keyboard layout visual guide and trainer";
          exec = "yxa-visual-guide";
          icon = "yxa-visual-guide";
          terminal = false;
          type = "Application";
          categories = [ "Utility" "Education" ];
          keywords = [ "keyboard" "layout" "trainer" "miryoku" ];
        };

        # Full visual guide package with desktop integration
        yxaVisualGuide = pkgs.stdenv.mkDerivation {
          pname = "yxa-visual-guide";
          version = "0.1.0";

          src = ./assets;
          layoutSrc = ./visual-guide/layouts;

          nativeBuildInputs = with pkgs; [
            librsvg
            makeWrapper
          ];

          dontBuild = true;

          installPhase = ''
            runHook preInstall

            # Install wrapped binary that uses bundled layout
            mkdir -p $out/bin
            makeWrapper ${yxaVisualGuideBin}/bin/yxa-visual-guide $out/bin/yxa-visual-guide \
              --add-flags "--file $out/share/yxa/layouts/miryoku-kbd-layout.vil" \
              --prefix LD_LIBRARY_PATH : "${pkgs.lib.makeLibraryPath [
                pkgs.libxkbcommon
                pkgs.libGL
                pkgs.wayland
                pkgs.xorg.libX11
                pkgs.xorg.libXcursor
                pkgs.xorg.libXrandr
                pkgs.xorg.libXi
                pkgs.vulkan-loader
              ]}"

            # Install layout files
            mkdir -p $out/share/yxa/layouts
            cp $layoutSrc/*.vil $out/share/yxa/layouts/

            # Install desktop entry
            mkdir -p $out/share/applications
            cp ${desktopItem}/share/applications/*.desktop $out/share/applications/

            # Convert SVG to PNG icons at various sizes
            for size in 16 24 32 48 64 128 256 512; do
              mkdir -p $out/share/icons/hicolor/''${size}x''${size}/apps
              ${pkgs.librsvg}/bin/rsvg-convert -w $size -h $size $src/logo.svg \
                -o $out/share/icons/hicolor/''${size}x''${size}/apps/yxa-visual-guide.png
            done

            # Install scalable SVG icon
            mkdir -p $out/share/icons/hicolor/scalable/apps
            cp $src/logo.svg $out/share/icons/hicolor/scalable/apps/yxa-visual-guide.svg

            runHook postInstall
          '';

          meta = with pkgs.lib; {
            description = "Yxa keyboard layout visual guide and trainer";
            license = licenses.mit;
            mainProgram = "yxa-visual-guide";
          };
        };

        # Firmware build script (vial-qmk with Docker)
        buildFirmware = pkgs.writeShellScriptBin "build-firmware" ''
          set -e
          FIRMWARE_DIR="''${FIRMWARE_DIR:-$PWD/firmware}"
          QMK_CACHE="$HOME/.cache/yxa-vial-qmk"
          KEYMAP="''${1:-miryoku}"

          echo "=== Building Yxa Firmware (keymap: $KEYMAP) ==="

          if ! command -v docker &> /dev/null; then
            echo "Error: Docker is not installed"
            exit 1
          fi

          # Check if cache exists and is a valid git repo
          if [ -d "$QMK_CACHE" ] && [ ! -d "$QMK_CACHE/.git" ]; then
            echo "Cache is corrupted (not a git repo), removing..."
            rm -rf "$QMK_CACHE"
          fi

          if [ ! -d "$QMK_CACHE" ]; then
            echo "Cloning vial-qmk (first time, ~500MB)..."
            ${pkgs.git}/bin/git clone --depth 1 https://github.com/vial-kb/vial-qmk.git "$QMK_CACHE"

            echo "Fetching submodules..."
            docker run --rm \
              --user "$(id -u):$(id -g)" \
              -e HOME=/qmk_firmware \
              -v "$QMK_CACHE:/qmk_firmware" \
              -w /qmk_firmware \
              ghcr.io/qmk/qmk_cli:latest \
              /bin/bash -c "git config --global --add safe.directory /qmk_firmware && git submodule update --init --recursive"
          else
            echo "Using cached vial-qmk at $QMK_CACHE"
            echo "  (delete $QMK_CACHE to force fresh clone)"

            # Check for missing submodules
            if [ ! -f "$QMK_CACHE/lib/chibios/os/hal/hal.mk" ]; then
              echo "Submodules incomplete, fetching..."
              docker run --rm \
                --user "$(id -u):$(id -g)" \
                -e HOME=/qmk_firmware \
                -v "$QMK_CACHE:/qmk_firmware" \
                -w /qmk_firmware \
                ghcr.io/qmk/qmk_cli:latest \
                /bin/bash -c "git config --global --add safe.directory /qmk_firmware && git submodule update --init --recursive"
            fi
          fi

          echo "Syncing keyboard files..."
          rm -rf "$QMK_CACHE/keyboards/yxa" 2>/dev/null || true
          cp -r "$FIRMWARE_DIR/keyboards/yxa" "$QMK_CACHE/keyboards/"

          echo "Building with Docker..."
          docker run --rm \
            --user "$(id -u):$(id -g)" \
            -e HOME=/qmk_firmware \
            -v "$QMK_CACHE:/qmk_firmware" \
            -w /qmk_firmware \
            ghcr.io/qmk/qmk_cli:latest \
            /bin/bash -c "git config --global --add safe.directory /qmk_firmware && make yxa:$KEYMAP"

          # Copy output
          mkdir -p "$FIRMWARE_DIR"
          if [ -f "$QMK_CACHE/yxa_''${KEYMAP}.bin" ]; then
            cp "$QMK_CACHE/yxa_''${KEYMAP}.bin" "$FIRMWARE_DIR/"
            echo ""
            echo "=== Build complete ==="
            echo "Firmware: $FIRMWARE_DIR/yxa_''${KEYMAP}.bin"
          elif [ -f "$QMK_CACHE/.build/yxa_''${KEYMAP}.bin" ]; then
            cp "$QMK_CACHE/.build/yxa_''${KEYMAP}.bin" "$FIRMWARE_DIR/"
            echo ""
            echo "=== Build complete ==="
            echo "Firmware: $FIRMWARE_DIR/yxa_''${KEYMAP}.bin"
          else
            echo ""
            echo "Build complete - check $QMK_CACHE/.build/ for output"
          fi
        '';

        # Flash firmware script
        flashFirmware = pkgs.writeShellScriptBin "flash-firmware" ''
          FIRMWARE="''${1:-firmware/yxa_miryoku.bin}"

          echo "=== Yxa Firmware Flash ==="
          echo ""

          if [ ! -f "$FIRMWARE" ]; then
            echo "Firmware file not found: $FIRMWARE"
            echo "Run 'build-firmware' first, or specify path as argument."
            exit 1
          fi

          echo "Firmware: $FIRMWARE"
          echo ""
          echo "IMPORTANT: Put keyboard in DFU mode first!"
          echo ""
          echo "To enter DFU mode:"
          echo "  1. Hold BOOT button on BlackPill"
          echo "  2. While holding, plug in USB (or tap RESET)"
          echo "  3. Release BOOT button"
          echo ""

          if ! ${pkgs.usbutils}/bin/lsusb | grep -qi "0483:df11\|STM.*DFU"; then
            echo "No DFU device detected!"
            echo "Put keyboard in DFU mode and try again."
            exit 1
          fi

          echo "DFU device found! Flashing..."
          ${pkgs.dfu-util}/bin/dfu-util -a 0 -s 0x08000000:leave -D "$FIRMWARE"

          echo ""
          echo "Done! Keyboard should restart automatically."
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
            pkgs.dfu-util
            pkgs.usbutils
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

          shellHook = ''
            echo ""
            echo "╔═══════════════════════════════════════════════╗"
            echo "║           Yxa Keyboard Development            ║"
            echo "╚═══════════════════════════════════════════════╝"
            echo ""
            echo "Commands:"
            echo "  visual-guide             - Run visual guide"
            echo "  build-firmware [keymap]  - Build firmware (default: miryoku)"
            echo "  flash-firmware [file]    - Flash firmware via DFU"
            echo "  fix-hid-perms            - Fix HID device permissions"
            echo ""

            alias visual-guide="cargo run --release --manifest-path visual-guide/Cargo.toml --"
          '';
        };

        apps = {
          visual-guide = {
            type = "app";
            program = "${yxaVisualGuide}/bin/yxa-visual-guide";
          };

          build-firmware = {
            type = "app";
            program = "${buildFirmware}/bin/build-firmware";
          };

          flash-firmware = {
            type = "app";
            program = "${flashFirmware}/bin/flash-firmware";
          };

          default = self.apps.${system}.visual-guide;
        };

        packages = {
          default = yxaVisualGuide;
          visual-guide = yxaVisualGuide;
          build-firmware = buildFirmware;
          flash-firmware = flashFirmware;
          fix-hid-perms = fixHidPerms;
        };
      }
    );
}
