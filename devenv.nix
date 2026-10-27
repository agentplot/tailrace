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

  # zot's own release build, for the tools layer's tests of the registry
  # store (crates/tailrace/tests/registry_store.rs): nixpkgs holds no zot at
  # devenv.lock's pin. The minimal build is the distribution specification
  # with the referrers API and none of zot's extensions. Linux takes the
  # build's dynamic loader from nixpkgs.
  zot =
    let
      version = "2.1.22";
      target =
        {
          aarch64-darwin = "darwin-arm64";
          x86_64-darwin = "darwin-amd64";
          aarch64-linux = "linux-arm64";
          x86_64-linux = "linux-amd64";
        }
        .${pkgs.stdenv.hostPlatform.system}
          or (throw "zot ${version}: no release build for ${pkgs.stdenv.hostPlatform.system}");
      hashes = {
        darwin-arm64 = "c81a26040ae79c7e8a90c306a2377c69982a86ff35c259427bd9accea0eca14c";
        darwin-amd64 = "3b18f445a8229372b0b521c07945dfd18a0a8ec1745601a22974f554f7614fb9";
        linux-arm64 = "4224c23122c43638f92875d6fb4f23564472d2584a348cb057440bcf4b43bf08";
        linux-amd64 = "a053169f97b4ceaa051e054d53971799188a398cb36acbaf5e08622ec8664c44";
      };
    in
    pkgs.stdenv.mkDerivation {
      pname = "zot";
      inherit version;
      src = pkgs.fetchurl {
        url = "https://github.com/project-zot/zot/releases/download/v${version}/zot-${target}-minimal";
        sha256 = hashes.${target};
      };
      dontUnpack = true;
      nativeBuildInputs = lib.optionals pkgs.stdenv.hostPlatform.isLinux [ pkgs.autoPatchelfHook ];
      installPhase = ''
        runHook preInstall
        install -Dm755 $src $out/bin/zot
        runHook postInstall
      '';
    };

  # Claude Code, run by the plugin's project (plugin-claude-code:validate) and
  # by root:marketplace, both through plugins/claude.sh: the release sluice's
  # plugins are checked with, so every host checks the plugin against one
  # engine, and never the user's own `claude`, which may be a wrapper adding
  # options of its own. Bump the version and the four checksums from the
  # release's manifest.zst.json, as sluice's devenv.nix does.
  claudeVersion = "2.1.296";
  claudeAssets = {
    aarch64-darwin = {
      platform = "darwin-arm64";
      sha256 = "26426aa3dcf41b0f554951766c3c583cf243345ca3d32fa590caf84685be5f97";
    };
    x86_64-darwin = {
      platform = "darwin-x64";
      sha256 = "6f1d6f926a303c423eb4815a01764165ed4375054bfa8ef6203e55fc806f50ac";
    };
    aarch64-linux = {
      platform = "linux-arm64";
      sha256 = "23aa91437a06eff80a65bfc32b0e4b5a6dda3e821019c543696c154274e56913";
    };
    x86_64-linux = {
      platform = "linux-x64";
      sha256 = "fac1d78be9446052ac084878c04c2648abb6b22d53279a685fa19ee517d74410";
    };
  };
  claudeAsset = claudeAssets.${pkgs.stdenv.hostPlatform.system};
  claude = pkgs.stdenvNoCC.mkDerivation {
    pname = "claude-code";
    version = claudeVersion;
    src = pkgs.fetchurl {
      url = "https://downloads.claude.ai/claude-code-releases/${claudeVersion}/${claudeAsset.platform}/claude.zst";
      inherit (claudeAsset) sha256;
    };
    dontUnpack = true;
    # The binary carries its program after the runtime; stripping drops it.
    dontStrip = true;
    nativeBuildInputs = [
      pkgs.zstd
      pkgs.makeBinaryWrapper
    ]
    ++ lib.optionals pkgs.stdenv.hostPlatform.isElf [ pkgs.autoPatchelfHook ];
    installPhase = ''
      mkdir -p $out/bin
      unzstd -q $src -o $out/bin/.claude
      chmod 755 $out/bin/.claude
      makeWrapper $out/bin/.claude $out/bin/claude \
        --set DISABLE_AUTOUPDATER 1 \
        --set DISABLE_INSTALLATION_CHECKS 1
    '';
  };

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

  # The git hook every worktree shares: prek's own shim, keeping its header
  # so prek takes it for its own. prek reads the config at the top of the
  # worktree a commit is in; a worktree no devenv shell has run in has no
  # link to it, as a place crew takes another unit's work into has not, and
  # there the shim hands prek the config it was built with. Its header's
  # last line holds the mark an install from an earlier devenv.nix looks
  # for, so a worktree still on one leaves this hook be.
  prekShim =
    type:
    pkgs.writeScript "prek-${type}" ''
      #!/bin/sh
      # File generated by prek: https://github.com/j178/prek
      # ID: 182c10f181da4464a3eec51b83331688
      # tailrace's devenv.nix: a worktree with no .pre-commit-config.yaml of its own reads the one this hook was built with.
      # An earlier install's mark: a worktree with no .pre-commit-config.yaml of its own reads main's

      HERE="$(cd "$(dirname "$0")" && pwd)"
      PREK="${lib.getExe config.git-hooks.package}"

      # Check if the full path to prek is executable, otherwise fallback to PATH
      if [ ! -x "$PREK" ]; then
          PREK="prek"
      fi

      if [ -e .pre-commit-config.yaml ]; then
          exec "$PREK" hook-impl --hook-dir "$HERE" --script-version 4 --hook-type=${type} -- "$@"
      fi
      exec "$PREK" hook-impl --hook-dir "$HERE" --script-version 4 --config ${config.git-hooks.configFile} --hook-type=${type} -- "$@"
    '';

  # The hooks are prekShim's, not devenv's: devenv 2.4 writes the installing
  # worktree's absolute config path into the hook, which every worktree
  # shares (cachix/devenv#3270), and prek's own install has no fallback.
  # Each hook is a link to its shim and a root of the store's collector, so
  # the prek and the config it names outlive every worktree that built
  # them. nix-store makes no root over a file that exists, so the hook goes
  # first. Installed into the repository it runs in: by `devenv shell`
  # into this one, and by check-hooks.sh into a scratch one.
  installHooks = pkgs.writeShellApplication {
    name = "install-hooks";
    runtimeInputs = [
      pkgs.git
      pkgs.nix
    ];
    text = ''
      hooks="$(git rev-parse --path-format=absolute --git-path hooks)"
    ''
    +
      lib.concatMapStrings
        (type: ''
          if [ "$(readlink "$hooks/${type}" || true)" != ${prekShim type} ]; then
            rm -f "$hooks/${type}"
            nix-store --add-root "$hooks/${type}" --realise ${prekShim type} >/dev/null
          fi
        '')
        [
          "pre-commit"
          "commit-msg"
        ];
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
    # check-compile-server.sh reads the host's sluice settings, TOML.
    pkgs.taplo
    # The tools layer: ssh-keygen signs the run records its tests stage as
    # sluice's signers sign them.
    pkgs.openssh
    # The tools layer's registries: the registry store's tests start zot
    # and the reference distribution, a registry with no referrers API, and
    # reach ECR through its credential helper.
    zot
    pkgs.distribution
    pkgs.amazon-ecr-credential-helper
    # The Claude Code plugin's checks (plugins/claude.sh).
    claude
  ]
  ++ lib.optional (sluice != null) sluice;

  env.PLAYWRIGHT_BROWSERS_PATH = "${pkgs.playwright-driver.browsers-chromium}";

  # The tools layer's tests run the pinned sluice, ssh-keygen and registries
  # by these paths: in a moon task, moon's Rust toolchain puts cargo's own bin
  # first on PATH, where a sluice built from source may be. Empty where
  # sluice releases nothing for the system, and the tests say so.
  env.TAILRACE_TOOLS_SLUICE = lib.optionalString (sluice != null) "${sluice}/bin/sluice";
  env.TAILRACE_TOOLS_SSH_KEYGEN = "${pkgs.openssh}/bin/ssh-keygen";
  env.TAILRACE_TOOLS_ZOT = "${zot}/bin/zot";
  env.TAILRACE_TOOLS_DISTRIBUTION = "${pkgs.distribution}/bin/registry";

  # moon's task archives (.moon/cache/outputs) are a link into the git common
  # dir, so every worktree shares them; the rest of .moon/cache records the
  # checkout's absolute paths and stays its own. moon's home is the
  # repository's too, so no moon of the user's runs in a task. In a sluice
  # run, which sources this before every command, cargo builds and the tests
  # run on the cores sluice handed the run (SLUICE_JOBS, sluice 0.37.0); a
  # session's own shell has no SLUICE_JOBS, and cargo takes every core.
  #
  # rustc compiles through the compile server and cache the host's sluice
  # settings declare, and this shell sets nothing of its own for them: sluice
  # hands a run the server's variables (sluice 0.43.0), none to a run holding
  # a check the policy marks cold, and a session's own shell inherits the
  # same from the host, which exports them into every shell. Where the host
  # declares none, rustc runs unwrapped (clause 133; check-compile-server.sh).
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

  # The commit hooks every worktree shares (installHooks), on entering the
  # shell and by hand.
  tasks."devenv:git-hooks:install".exec = lib.mkForce "${installHooks}/bin/install-hooks";
  scripts.install-hooks = {
    description = "The commit hooks, into the repository it runs in";
    exec = ''exec ${installHooks}/bin/install-hooks "$@"'';
  };

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
