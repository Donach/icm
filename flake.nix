{
  description = "ICM — Infinite Context Memory: permanent memory for AI agents";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

  outputs =
    { self, nixpkgs }:
    let
      # x86_64-linux is what we actually build and test; aarch64-linux uses
      # the same nixpkgs dependencies and is expected to work but is not
      # verified. darwin likely works too (onnxruntime and openssl exist
      # there) — users of those systems are welcome to verify and extend.
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      forAll = nixpkgs.lib.genAttrs systems;
    in
    {
      packages = forAll (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};

          # One parameterised builder for every feature flavour. NOTE: do not
          # try to change features via `overrideAttrs` on the result —
          # buildRustPackage consumes `buildFeatures`/`buildNoDefaultFeatures`
          # when it is *called*, so a post-hoc override silently produces a
          # default-features binary under the new name.
          mkIcm =
            {
              pname,
              features ? null, # null = default features
              description,
            }:
            pkgs.rustPlatform.buildRustPackage {
              inherit pname;
              # Read straight from the crate manifest so release-please bumps
              # can never drift from the flake.
              version = (nixpkgs.lib.importTOML ./crates/icm-cli/Cargo.toml).package.version;

              src = ./.;
              # Vendor the crate deps straight from the committed lockfile —
              # no cargoHash to update on every dependency bump.
              cargoLock.lockFile = ./Cargo.lock;

              buildNoDefaultFeatures = features != null;
              buildFeatures = if features == null then [ ] else features;
              cargoBuildFlags = [
                "-p"
                "icm-cli"
              ];

              nativeBuildInputs = [ pkgs.pkg-config ];
              buildInputs = [
                pkgs.openssl
                pkgs.onnxruntime
              ];

              env = {
                # Link the system openssl instead of compiling the vendored copy.
                OPENSSL_NO_VENDOR = "1";
                # fastembed -> ort: use the nixpkgs onnxruntime instead of a
                # downloaded prebuilt (which wouldn't work in the sandbox).
                ORT_STRATEGY = "system";
                ORT_LIB_LOCATION = "${pkgs.lib.getLib pkgs.onnxruntime}/lib";
              };

              # The test suite runs in CI via cargo; the sandboxed nix build
              # skips it (some tests need writable HOME / model downloads).
              doCheck = false;

              # onnxruntime (system ORT) is built against this nixpkgs' gcc
              # libstdc++, newer than the one the Rust toolchain puts first on
              # the RPATH; without help the binary can abort at startup with
              # `GLIBCXX_3.4.xx not found (required by libonnxruntime.so)`.
              # Extend the RPATH (instead of an LD_LIBRARY_PATH wrapper, which
              # would leak into every child process the binary spawns).
              postFixup = ''
                patchelf --add-rpath ${
                  pkgs.lib.makeLibraryPath [
                    pkgs.stdenv.cc.cc.lib
                    pkgs.onnxruntime
                  ]
                } $out/bin/icm
              '';

              meta = {
                inherit description;
                homepage = "https://github.com/rtk-ai/icm";
                license = nixpkgs.lib.licenses.asl20;
                mainProgram = "icm";
              };
            };
        in
        {
          icm = mkIcm {
            pname = "icm";
            description = "Permanent memory for AI agents. Single binary, zero dependencies, MCP native.";
          };
          # Same source, libSQL/Turso backend instead of rusqlite. A separate
          # package output because the two backends cannot link into one
          # binary today: libsql-ffi and libsqlite3-sys both bundle sqlite3
          # and their symbols collide (see docs/turso-backend.md).
          icm-turso = mkIcm {
            pname = "icm-turso";
            features = [
              "turso"
              "embeddings"
              "tui"
            ];
            description = "ICM with the opt-in libSQL/Turso backend (concurrent multi-writer memory)";
          };
          default = self.packages.${system}.icm;
        }
      );

      apps = forAll (system: {
        default = {
          type = "app";
          program = "${self.packages.${system}.icm}/bin/icm";
        };
      });

      devShells = forAll (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
        in
        {
          default = pkgs.mkShell {
            packages = with pkgs; [
              cargo
              rustc
              rustfmt
              clippy
              rust-analyzer
              pkg-config
              openssl
              onnxruntime
            ];
            env = {
              OPENSSL_NO_VENDOR = "1";
              ORT_STRATEGY = "system";
              ORT_LIB_LOCATION = "${pkgs.lib.getLib pkgs.onnxruntime}/lib";
            };
          };
        }
      );
    };
}
