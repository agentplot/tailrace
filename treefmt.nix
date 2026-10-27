_: {
  # A tracked file no formatter claims and no exclude names fails the check
  # (core.25): a new file type is decided in the change that adds it.
  settings.on-unmatched = "fatal";

  # What no formatter owns, each with its reason (core.25).
  settings.excludes = [
    # Prose: no formatter here rewraps it.
    "*.md"
    "*.mdx"
    # Written by cargo, devenv and bun in their own layouts.
    "*.lock"
    # Drawn images and rendered diagrams; no formatter keeps them stable.
    "*.svg"
    "*.png"
    "*.ico"
    "*.webp"
    # The site's components: Biome formats only an .astro file's
    # frontmatter, and only experimentally.
    "site/**/*.astro"
    # The site's mock pages, kept as drawn; Biome's HTML support is
    # experimental.
    "site/mock/**"
    # The licence, as the licence is written.
    "LICENSE"
    # Empty markers that keep OpenSpec's directories in the tree.
    "*.gitkeep"
    # Build outputs and caches.
    "target/**"
    ".devenv/**"
    ".moon/cache/**"
    # Claude Code rewrites it in its own layout when a setting changes.
    ".claude/settings.json"
    # The herdr plugin's jq programs: no formatter or linter for jq in the
    # toolchain; plugin-herdr's tests run every program under sh and dash.
    "plugins/herdr/lib/*.jq"
    # The herdr plugin's documents and frames, compared byte for byte.
    "plugins/herdr/tests/docs/**"
  ];

  programs.rustfmt.enable = true;
  programs.rustfmt.edition = "2021";

  programs.deadnix.enable = true;
  programs.nixfmt.enable = true;
  settings.formatter.deadnix.pipeline = "nix";
  settings.formatter.deadnix.priority = 1;
  settings.formatter.nixfmt.pipeline = "nix";
  settings.formatter.nixfmt.priority = 2;

  programs.shellcheck.enable = true;
  programs.shfmt.enable = true;
  programs.shfmt.indent_size = 2;
  settings.formatter.shellcheck.pipeline = "shell";
  settings.formatter.shellcheck.priority = 1;
  settings.formatter.shfmt.pipeline = "shell";
  settings.formatter.shfmt.priority = 2;
  # The herdr plugin's stand-ins: #!/bin/sh scripts with no extension.
  settings.formatter.shellcheck.includes = [ "plugins/herdr/tests/fake/*" ];
  settings.formatter.shfmt.includes = [ "plugins/herdr/tests/fake/*" ];

  # .prototools is TOML and .versionrc convco's YAML, under names with no
  # extension.
  programs.taplo.enable = true;
  settings.formatter.taplo.includes = [ ".prototools" ];
  programs.yamlfmt.enable = true;
  settings.formatter.yamlfmt.includes = [ ".versionrc" ];
  programs.yamlfmt.settings.formatter = {
    type = "basic";
    retain_line_breaks_single = true;
  };

  # Biome formats and lints the TypeScript, JavaScript, CSS and JSON in
  # one pass (`biome check`), its recommended rules and nothing more; a
  # lint error fails `treefmt --ci`. Its settings are here, not in a
  # biome.json, since treefmt-nix hands Biome a generated one.
  programs.biome.enable = true;
  programs.biome.formatCommand = "check";
  programs.biome.settings = {
    formatter = {
      indentStyle = "space";
      indentWidth = 2;
      lineWidth = 100;
    };
    javascript.formatter = {
      quoteStyle = "double";
      trailingCommas = "all";
      semicolons = "always";
    };
    json.formatter.trailingCommas = "none";
    linter.rules.recommended = true;
  };
  # actionlint reads the workflows' keys and expressions, which yamlfmt
  # only lays out.
  programs.actionlint.enable = true;
}
