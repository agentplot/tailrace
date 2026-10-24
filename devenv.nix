{
  pkgs,
  lib,
  config,
  ...
}:

let
  # proto's own release build: nixpkgs trails it by minor versions or more.
  # Linux takes the static musl build, so no loader needs patching.
  proto =
    let
      version = "0.63.1";
      target =
        {
          aarch64-darwin = "aarch64-apple-darwin";
          x86_64-darwin = "x86_64-apple-darwin";
          aarch64-linux = "aarch64-unknown-linux-musl";
          x86_64-linux = "x86_64-unknown-linux-musl";
        }
        .${pkgs.stdenv.hostPlatform.system}
          or (throw "proto ${version}: no release build for ${pkgs.stdenv.hostPlatform.system}");
      hashes = {
        aarch64-apple-darwin = "8bfde4c862381eeb6e82d0858bb6215732cbd1e384e4eae45da8fa08095e0ffd";
        x86_64-apple-darwin = "aaaff78c7125de1b765f00177f35da75ee16fb1363e07632ace27213ff5d893b";
        aarch64-unknown-linux-musl = "111f5b6a38d3979ebafbc095c8d91ef69c99d1f018addb3582179d4ab91b3441";
        x86_64-unknown-linux-musl = "8596b98ccd833d1203e71931ba25e866d3239f9d3859c922a035a7b830f29d17";
      };
    in
    pkgs.stdenvNoCC.mkDerivation {
      pname = "proto_cli";
      inherit version;
      src = pkgs.fetchurl {
        url = "https://github.com/moonrepo/proto/releases/download/v${version}/proto_cli-${target}.tar.xz";
        sha256 = hashes.${target};
      };
      sourceRoot = "proto_cli-${target}";
      installPhase = ''
        runHook preInstall
        install -Dm755 -t $out/bin proto proto-shim
        runHook postInstall
      '';
    };

  # sluice's own release build, for the tools layer's tests of the sluice
  # landing (crates/tailrace/tests/landing_sluice.rs): the version tailrace
  # reads the records of. Linux takes the build's dynamic loader and
  # libraries from nixpkgs. On a system sluice releases nothing for,
  # sluice is not in the shell, and those tests fail at their first step
  # naming it.
  sluice =
    let
      version = "0.37.0";
      system = pkgs.stdenv.hostPlatform.system;
      hashes = {
        aarch64-darwin = "bd34de2c1768c45143da4349899dab6a9e19e3dac1f8eca022f6f2fec6b60fb1";
        aarch64-linux = "663c2e87d8bcc9027d53713e609b8813cff8258baff48ca04aed11f8abb8bdeb";
        x86_64-linux = "d137876bea447e04f6d83527fa7bfd060ecb35856a357a48d6d8032c36cc7848";
      };
    in
    if hashes ? ${system} then
      pkgs.stdenv.mkDerivation {
        pname = "sluice";
        inherit version;
        src = pkgs.fetchurl {
          url = "https://github.com/agentplot/sluice/releases/download/v${version}/sluice-${system}.tar.gz";
          sha256 = hashes.${system};
        };
        sourceRoot = ".";
        nativeBuildInputs = lib.optionals pkgs.stdenv.hostPlatform.isLinux [ pkgs.autoPatchelfHook ];
        buildInputs = lib.optionals pkgs.stdenv.hostPlatform.isLinux [ pkgs.stdenv.cc.cc.lib ];
        installPhase = ''
          runHook preInstall
          install -Dm755 -t $out/bin sluice
          runHook postInstall
        '';
      }
    else
      null;

  # The application toolchain .prototools pins, on PATH. proto's home is
  # this repository's, in the git common dir so every worktree shares it,
  # and only this repository's pins are read (no global .prototools).
  protoHome = ''
    PROTO_HOME="$(${lib.getExe pkgs.git} rev-parse --path-format=absolute --git-common-dir)/proto"
    export PROTO_HOME
  '';
  activate = ''
    ${protoHome}
    eval "$(${proto}/bin/proto activate bash --export --no-bin --config-mode upwards 2>/dev/null)"
  '';

  # The lint gate.
  lint = pkgs.writeShellApplication {
    name = "lint";
    runtimeInputs = [ pkgs.git ];
    text = ''
      cd "$(git rev-parse --show-toplevel)"
      ${activate}
      cargo clippy --workspace --all-targets --all-features -- -D warnings
    '';
  };

  # treefmt's rustfmt: the pinned toolchain's.
  rustfmt = pkgs.writeShellApplication {
    name = "rustfmt";
    text = ''
      ${activate}
      exec rustfmt "$@"
    '';
  };
in
{
  # The system tools are this file's, pinned by devenv.lock; the application
  # tools (moon, Rust) are .prototools', installed by the
  # tailrace:proto:install task on entering the shell and activated by
  # enterShell. Every cargo and moon command in this repository runs inside
  # `devenv shell`.
  packages = [
    pkgs.git
    pkgs.jq
    pkgs.git-cliff
    proto
    # The site (site/): bun builds it, and Playwright drives the headless
    # Chromium its screenshots are taken in (site/scripts/shots.mjs).
    pkgs.bun
    pkgs.playwright-test
    # The tools layer: ssh-keygen signs the run records its tests stage as
    # sluice's signers sign them.
    pkgs.openssh
  ]
  ++ lib.optional (sluice != null) sluice;

  env.PLAYWRIGHT_BROWSERS_PATH = "${pkgs.playwright-driver.browsers-chromium}";

  # moon's task archives (.moon/cache/outputs) are a link into the git common
  # dir, so every worktree shares them; the rest of .moon/cache records the
  # checkout's absolute paths and stays its own. moon's home is the
  # repository's too, so no moon of the user's runs in a task. In a sluice
  # run, which sources this before every command, cargo builds and the tests
  # run on the cores sluice handed the run (SLUICE_JOBS, sluice 0.37.0); a
  # session's own shell has no SLUICE_JOBS, and cargo takes every core.
  enterShell = ''
    if [ -n "''${SLUICE_JOBS:-}" ]; then
      CARGO_BUILD_JOBS="$SLUICE_JOBS"
      RUST_TEST_THREADS="$SLUICE_JOBS"
      export CARGO_BUILD_JOBS RUST_TEST_THREADS
    fi
    if _moon_root="$(git rev-parse --show-toplevel 2>/dev/null)" &&
      _moon_common="$(git rev-parse --path-format=absolute --git-common-dir 2>/dev/null)"; then
      _moon_cache="$_moon_root/.moon/cache"
      [ -L "$_moon_cache" ] && rm -f "$_moon_cache"
      mkdir -p "$_moon_common/moon-outputs" "$_moon_cache"
      if [ ! -L "$_moon_cache/outputs" ]; then
        rm -rf "$_moon_cache/outputs"
        ln -s "$_moon_common/moon-outputs" "$_moon_cache/outputs"
      fi
      MOON_HOME="$_moon_common/moon-home"
      export MOON_HOME
    fi
    unset _moon_root _moon_common _moon_cache
    ${activate}
    ${lib.optionalString (sluice != null) ''
      # proto puts cargo's own bin first, where a sluice built from source
      # may be: the tree's pin comes before it.
      PATH="${sluice}/bin:$PATH"
      export PATH
    ''}
  '';

  # The tool versions .prototools lists, installed before enterShell
  # activates them. A task, not part of enterShell, so a script that only
  # sources the shell's environment never installs.
  tasks."tailrace:proto:install" = {
    exec = ''
      ${protoHome}
      ${proto}/bin/proto install --config-mode upwards
    '';
    before = [ "devenv:enterShell" ];
  };

  treefmt.enable = true;
  treefmt.config = {
    imports = [ ./treefmt.nix ];
    # The tree is found from where treefmt runs, by this file, rather than
    # devenv's evaluated root written into the wrapper: with that, a
    # wrapper evaluated elsewhere checks the other tree.
    projectRootFile = "devenv.nix";
    programs.rustfmt.package = rustfmt;
  };

  # The floor under every commit, installed into the common hooks directory
  # on `devenv shell`. convco refuses a message that is not a conventional
  # commit (.versionrc), since git-cliff releases from them.
  git-hooks.hooks = {
    convco = {
      enable = true;
      settings.configPath = ".versionrc";
    };
    treefmt.enable = true;
    clippy-gate = {
      enable = true;
      name = "clippy (-D warnings)";
      entry = "${lint}/bin/lint";
      files = "\\.rs$|Cargo\\.(toml|lock)$";
      pass_filenames = false;
    };
  };

  scripts.lint = {
    description = "The lint gate: cargo clippy -D warnings";
    exec = ''exec ${lint}/bin/lint "$@"'';
  };
  # What CI would release from here: the next version and its notes.
  scripts.release-preview = {
    description = "The next release's version and notes: git cliff --unreleased --bump";
    exec = ''
      set -euo pipefail
      cd "$(git rev-parse --show-toplevel)"
      git cliff --unreleased --bump --strip all "$@"
    '';
  };

  # devenv 2.4 writes the worktree's absolute config path into the hook,
  # which every worktree shares; a plain `prek install` writes none, and prek
  # reads the config at the top of the worktree a commit is in. Remove once
  # devenv installs without the path (cachix/devenv#3270).
  tasks."devenv:git-hooks:install".exec = lib.mkForce ''
    set -euo pipefail
    hooks="$(${lib.getExe pkgs.git} rev-parse --path-format=absolute --git-path hooks)"
    for type in pre-commit commit-msg; do
      if ! grep -q 'File generated by prek' "$hooks/$type" 2>/dev/null || grep -q -- '--config=' "$hooks/$type"; then
        ${lib.getExe config.git-hooks.package} install --force -t "$type"
      fi
    done
  '';

  # `devenv shell -- site-dev`: the site at http://localhost:4321/tailrace/.
  scripts.site-dev = {
    description = "The site, served with live reload: astro dev";
    exec = ''
      set -euo pipefail
      cd "$(git rev-parse --show-toplevel)/site"
      bun install --frozen-lockfile
      exec bun run dev "$@"
    '';
  };
}
