{
  description = "SZR35 Miryoku Keyboard - Overlay, Firmware, and Tools";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        # Python environment for the overlay
        pythonEnv = pkgs.python3.withPackages (ps: with ps; [
          pyqt6
          hidapi
          evdev
          rich  # for terminal trainer
        ]);

      in {
        # Development shell
        devShells.default = pkgs.mkShell {
          buildInputs = [
            pythonEnv
            pkgs.dfu-util
            pkgs.docker
            pkgs.usbutils
          ];

          shellHook = ''
            echo "SZR35 Miryoku Development Environment"
            echo ""
            echo "Commands:"
            echo "  overlay     - Run the layer overlay"
            echo "  trainer     - Run the terminal trainer"
            echo "  flash       - Flash firmware (keyboard must be in DFU mode)"
            echo "  build       - Build firmware using Docker"
            echo ""
            echo "HID Permissions:"
            echo "  If overlay can't access keyboard, run:"
            echo "  sudo chmod 666 /dev/hidraw*"
            echo ""

            # Aliases
            alias overlay="python $PWD/overlay/miryoku_overlay.py"
            alias trainer="python $PWD/overlay/miryoku_trainer.py"
            alias trainer-hid="python $PWD/overlay/miryoku_trainer.py --hid"
            alias flash="sudo dfu-util -a 0 -s 0x08000000:leave -D firmware/szrkbd_szr35_vial.bin"
            alias build="docker run --rm -v /home/draxel/Downloads/vial-qmk-szr35:/qmk_firmware -w /qmk_firmware qmkfm/qmk_cli:latest make szrkbd/szr35:vial && cp /home/draxel/Downloads/vial-qmk-szr35/szrkbd_szr35_vial.bin $PWD/firmware/"
          '';
        };

        # Overlay application
        apps.overlay = {
          type = "app";
          program = toString (pkgs.writeShellScript "overlay" ''
            ${pythonEnv}/bin/python ${self}/overlay/miryoku_overlay.py "$@"
          '');
        };

        # Terminal trainer
        apps.trainer = {
          type = "app";
          program = toString (pkgs.writeShellScript "trainer" ''
            ${pythonEnv}/bin/python ${self}/overlay/miryoku_trainer.py "$@"
          '');
        };

        # Flash firmware
        apps.flash = {
          type = "app";
          program = toString (pkgs.writeShellScript "flash" ''
            echo "Put keyboard in DFU mode (short boot pads while plugging in USB)"
            echo "Press Enter when ready..."
            read
            sudo ${pkgs.dfu-util}/bin/dfu-util -a 0 -s 0x08000000:leave -D ${self}/firmware/szrkbd_szr35_vial.bin
          '');
        };

        apps.default = self.apps.${system}.overlay;
      }
    );
}
