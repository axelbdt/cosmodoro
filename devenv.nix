{
  pkgs,
  lib,
  config,
  inputs,
  ...
}:

let
  dlopenLibraries = with pkgs; [
    libGL
    wayland
    libxkbcommon
    alsa-lib
  ];
  claudeCodePackages = with pkgs; [
    claude-code
    socat
    bubblewrap
  ];
in
{
  # https://devenv.sh/basics/

  # https://devenv.sh/packages/
  packages = with pkgs; [ ] ++ dlopenLibraries ++ claudeCodePackages;

  # https://devenv.sh/languages/
  languages.rust.enable = true;
  languages.rust.rustflags = "-C link-arg=-Wl,-rpath,${pkgs.lib.makeLibraryPath dlopenLibraries}";

  # https://devenv.sh/processes/
  # processes.dev.exec = "${lib.getExe pkgs.watchexec} -n -- ls -la";

  # https://devenv.sh/services/
  # services.postgres.enable = true;

  # https://devenv.sh/tasks/
  # tasks = {
  #   "myproj:setup".exec = "mytool build";
  #   "devenv:enterShell".after = [ "myproj:setup" ];
  # };

  # https://devenv.sh/tests/
  enterTest = ''
    echo "Running tests"
    git --version | grep --color=auto "${pkgs.git.version}"
  '';

  # https://devenv.sh/git-hooks/
  # git-hooks.hooks.shellcheck.enable = true;

  # See full reference at https://devenv.sh/reference/options/

  claude.code.enable = true;
  git-hooks.hooks = {
    rustfmt.enable = true;
    nixfmt.enable = true;
  };
}
