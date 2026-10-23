# tailrace as a nix package, for `nix profile install github:agentplot/tailrace`
# and for the GHA build. The development shell is devenv's (devenv.nix);
# this flake only builds and installs. Rust is the version .prototools
# names, so the release is built by the compiler the checks ran with. The
# package's version is the release tag's: CI builds with
# `TAILRACE_VERSION=<version> nix build --impure`; any other build is
# 0.0.0, Cargo.toml's. The binary itself names the revision it was built
# from, never a version (statement 24, C.11).
{
  description = "tailrace: build once, sign everything, promote with proof, in your own account";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
    }:
    let
      systems = [
        "aarch64-darwin"
        "x86_64-darwin"
        "aarch64-linux"
        "x86_64-linux"
      ];
      forAll =
        f:
        nixpkgs.lib.genAttrs systems (
          system:
          f (
            import nixpkgs {
              inherit system;
              overlays = [ rust-overlay.overlays.default ];
            }
          )
        );
      rustVersion = (builtins.fromTOML (builtins.readFile ./.prototools)).rust;
      workspace = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).workspace.package;
      released = builtins.getEnv "TAILRACE_VERSION";
      version = if released != "" then released else workspace.version;
      revision = self.rev or self.dirtyRev or "unknown";
    in
    {
      packages = forAll (
        pkgs:
        let
          rust = pkgs.rust-bin.stable.${rustVersion}.minimal;
          platform = pkgs.makeRustPlatform {
            cargo = rust;
            rustc = rust;
          };
        in
        {
          default = platform.buildRustPackage {
            pname = "tailrace";
            inherit version;
            env.TAILRACE_REVISION = revision;
            src = self;
            cargoLock.lockFile = ./Cargo.lock;
            cargoBuildFlags = [
              "-p"
              "tailrace"
            ];
            # The tests run in the devenv shell, through moon, by layer.
            doCheck = false;
            meta = {
              description = "Build once, sign everything, promote with proof, in your own account";
              homepage = "https://github.com/agentplot/tailrace";
              license = pkgs.lib.licenses.mit;
              mainProgram = "tailrace";
            };
          };
        }
      );
    };
}
