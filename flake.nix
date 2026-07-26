{
  description = "Cross-platform LaunchBox and BigBox port workspace";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    # Nixpkgs unstable dropped Intel macOS in 26.11. Keep that still-supported
    # host on the 26.05 Darwin branch while Apple Silicon follows unstable.
    nixpkgs-intel-darwin.url = "github:NixOS/nixpkgs/nixpkgs-26.05-darwin";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      nixpkgs-intel-darwin,
      flake-utils,
      rust-overlay,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        nixpkgsForSystem = if system == "x86_64-darwin" then nixpkgs-intel-darwin else nixpkgs;
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgsForSystem { inherit system overlays; };
        lib = pkgs.lib;

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [
            "clippy"
            "rust-analyzer"
            "rust-src"
            "rustfmt"
          ];
          targets = [
            "aarch64-apple-darwin"
            "x86_64-apple-darwin"
            "x86_64-pc-windows-gnu"
          ];
        };

        qtModules =
          with pkgs.qt6;
          [
            qt5compat
            qtbase
            qtdeclarative
            qtimageformats
            qtmultimedia
            qtquick3d
            qtsvg
            qttools
          ]
          ++ lib.optionals pkgs.stdenv.isLinux [ qtwayland ];

        # CXX-Qt asks qmake for one coherent Qt prefix. qt6.env provides that
        # merged view instead of exposing unrelated package store paths.
        qtEnv = pkgs.qt6.env "launchbox-port-qt-env" qtModules;

        nativeBuildInputs =
          with pkgs;
          [
            rustToolchain
            cmake
            ninja
            pkg-config
            dosbox-staging
            p7zip
            ripgrep
            scummvm
            sqlite
          ]
          ++ lib.optionals pkgs.stdenv.isLinux [
            appimage-run
            imagemagick
            xvfb-run
          ];

        buildInputs = qtModules ++ [ pkgs.libGL ];

        cargoWrapped = pkgs.writeShellScriptBin "cargo" ''
          export PATH="${qtEnv}/bin:${qtEnv}/libexec:$PATH"
          export QMAKE="${qtEnv}/bin/qmake"
          exec ${rustToolchain}/bin/cargo "$@"
        '';

        cleanSource = lib.cleanSourceWith {
          src = ./.;
          filter =
            path: type:
            let
              rel = lib.removePrefix "${toString ./.}/" (toString path);
            in
            !(
              rel == ".git"
              || lib.hasPrefix ".git/" rel
              || rel == "decompiled"
              || lib.hasPrefix "decompiled/" rel
              || rel == "oracle"
              || lib.hasPrefix "oracle/" rel
              || rel == "target"
              || lib.hasPrefix "target/" rel
            );
        };
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "launchbox-port";
          version = "0.1.0";
          src = cleanSource;

          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = nativeBuildInputs ++ [ pkgs.qt6.wrapQtAppsHook ];
          inherit buildInputs;

          dontUseCmakeConfigure = true;
          dontUseNinjaBuild = true;
          dontUseNinjaInstall = true;

          QMAKE = "${qtEnv}/bin/qmake";
          qtWrapperArgs = [
            "--prefix PATH : ${
              lib.makeBinPath [
                pkgs.dosbox-staging
                pkgs.p7zip
                pkgs.scummvm
              ]
              + lib.optionalString pkgs.stdenv.isLinux ":${lib.makeBinPath [ pkgs.appimage-run ]}"
            }"
          ];

          preBuild = ''
            export QMAKE="${qtEnv}/bin/qmake"
            export PATH="${qtEnv}/bin:${qtEnv}/libexec:$PATH"
            export QT_INCLUDE_PATH="${qtEnv}/include"
            export QT_LIBEXEC_PATH="${qtEnv}/libexec"
          '';

          cargoBuildFlags = [
            "--package"
            "lb-shell"
          ];
          doCheck = true;
          checkPhase = ''
            runHook preCheck
            cargo test --workspace --all-targets --release
            cargo build --package lb-process-fixture --release \
              --target ${pkgs.stdenv.hostPlatform.rust.rustcTarget}
            bash ./scripts/check_qml.sh
            runHook postCheck
          '';
          postFixup = ''
            for frontend in launchbox bigbox; do
              if ! grep -a -F '${pkgs.p7zip}/bin' "$out/bin/$frontend" >/dev/null; then
                echo "$frontend wrapper is missing the packaged 7-Zip PATH" >&2
                exit 1
              fi
              if ! grep -a -F '${pkgs.dosbox-staging}/bin' "$out/bin/$frontend" >/dev/null; then
                echo "$frontend wrapper is missing the packaged DOSBox Staging PATH" >&2
                exit 1
              fi
              if ! grep -a -F '${pkgs.scummvm}/bin' "$out/bin/$frontend" >/dev/null; then
                echo "$frontend wrapper is missing the packaged ScummVM PATH" >&2
                exit 1
              fi
              ${lib.optionalString pkgs.stdenv.isLinux ''
                if ! grep -a -F '${pkgs.appimage-run}/bin' "$out/bin/$frontend" >/dev/null; then
                  echo "$frontend wrapper is missing the packaged appimage-run PATH" >&2
                  exit 1
                fi
              ''}
            done
          '';

          meta = with lib; {
            description = "Native Rust and Qt port of the LaunchBox/BigBox experience";
            license = licenses.agpl3Only;
            mainProgram = "launchbox";
            platforms = platforms.linux ++ platforms.darwin;
          };
        };

        checks.default = self.packages.${system}.default;

        devShells.default = pkgs.mkShell {
          inherit nativeBuildInputs buildInputs;

          packages = [
            cargoWrapped
            pkgs.git
            pkgs.nixfmt
            rustToolchain
          ];

          FLAKE_INPUTS = builtins.concatStringsSep ":" [
            "${nixpkgs}"
            "${nixpkgs-intel-darwin}"
            "${rust-overlay}"
            "${flake-utils}"
          ];

          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
          RUSTC = "${rustToolchain}/bin/rustc";
          CARGO = "${cargoWrapped}/bin/cargo";
          QMAKE = "${qtEnv}/bin/qmake";
          QT_QPA_PLATFORM = lib.optionalString pkgs.stdenv.isLinux "wayland;xcb";

          shellHook = ''
            export PATH="${cargoWrapped}/bin:${rustToolchain}/bin:${qtEnv}/bin:${qtEnv}/libexec:$PATH"
            export RUSTC="${rustToolchain}/bin/rustc"
            export QMAKE="${qtEnv}/bin/qmake"
            export QT_INCLUDE_PATH="${qtEnv}/include"
            export QT_LIBEXEC_PATH="${qtEnv}/libexec"
            echo "LaunchBox port development environment"
            echo "Qt: $(qmake -query QT_VERSION)"
            echo "Rust: $("$RUSTC" --version)"
            echo "Run 'cargo test --workspace' or 'cargo run -p lb-shell --bin launchbox'"
          '';
        };

        formatter = pkgs.nixfmt;
      }
    );
}
