# Bow - Stow on steriods
 > It's pronounced "bow" as in like, the archery thing

## Features

- Declaratively manage user-level packages from many different sources (Flatpak, AppImage, your system package manager, or just raw binaries!)
- Manage your dotfiles via symlinks
    - Drop in replacement for [GNU Stow](https://www.gnu.org/software/stow/) (this is a huge goal)
- Modes to either just install what you list or idempotent-ly manage your packages by removing any (user) packages that are installed but not listed in your config
- Super simple config through a single YAML file
- Asynchronous installation of packages (HUUUUUUGE todo...)
- Oh and it's written in rust :3

## Supported package providers

> This is a checklist for now

- [x] Flatpak (basically fully working, but more features to come perchance)
- [ ] Binaries (in progress)
- [ ] AppImage
- [ ] System package management
- [ ] Snap maybe? Idk I've literally never used snaps
- [ ] Dotfiles (symlinking files to `$HOME`)

## Usage

> (very WIP, more like "planned usage", doesn't quite work like this yet)

- Create a file named `bow.yaml` with contents as per the below example

```yaml
# `imperative` or `idempotent`
#   imperative will simply attempt to install the packges declared in this file
#   idempotent will prompt to remove any user-level packages *not* declared in this file
mode: imperative

# define your packages here under their respective provider
packages:
    # a list of flatpak IDs
    flatpak:
        - app.zen_browser.zen
        - dev.zed.Zed

    # raw binaries, define the default install folder and then binaries follow
    #   package entry -
    #     (required)  name: name of the final binary/used for identification
    #     (required)  url: source URL to pull the binary from (can use {{ version }} which will be substituted in)
    #     (optional)  sum: checksum URL to pull the binary checksum file from (can use {{ version }} which will be substituted in)
    #     (optional*) version: version of the binary, required if {{ version }} is used in either `url` or `sum`, can be any arbitrary string
    binary:
        install_folder: $HOME/.local/bin
        packages:
            - name: kubectl
              url: https://dl.k8s.io/release/{{ version }}/bin/linux/amd64/kubectl
              sum: https://dl.k8s.io/release/{{ version }}/bin/linux/amd64/kubectl.sha256
              version: v1.34.1
```

- Run bow with the above yaml

```sh
bow bow.yaml
```
