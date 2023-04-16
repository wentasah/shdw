# shdw: Manage symlinks to files in a shadow directory

Do you accidentally delete important and uncommitted files, such as
`.envrc`, `.dit-locals.el` by running `git clean -fxd`? If yes, `shdw`
might help.

## Installation

    cargo install --path=.

Don't forgot to add `~/.cargo/bin` to `PATH`.

### Nix

    nix profile install github:wentasah/shdw

This will install not only the `shdw` binary, but also shell
completion files and a man page.

## Usage

1. Add every important file to a shadow directory:

        $ shdw add .envrc

2. After accidental deletion, run:

        $ shdw restore

If you want to restore files automatically after `git clean`, you can
define the following function in your shell startup file (`.bashrc`,
`.zshrc`, etc.)

```sh
git() {
        case "$1" in
            clean) shift; shdw git-clean "$@";;
            *) command git "$@";;
        esac
}
```

## Command line reference

<!-- `$ shdw --help` -->
```
Manage symlinks to files in a shadow directory

Usage: shdw [OPTIONS] <COMMAND>

Commands:
  add        Move given files to the shadow directory and create symlinks to them in their original location
  ls         List files in the shadow directory
  restore    (Re)create symlinks to all files in the shadow directory
  rm         Move given files out of the shadow directory to the place of their symlinks at or under the current directory
  git-clean  Run 'git clean' with given GIT_OPTIONS followed by 'shdw restore'
  help       Print this message or the help of the given subcommand(s)

Options:
      --shadow-dir <SHADOW_DIR>  Shadow directory to use [env: SHDW_DIR=] [default: /home/wsh/.config/shdw/dir]
      --base-dir <BASE_DIR>      Base directory [env: SHDW_BASE_DIR=] [default: /home/wsh]
  -h, --help                     Print help
  -V, --version                  Print version
```

<!-- Local Variables: -->
<!-- compile-command: "mdsh" -->
<!-- End: -->
