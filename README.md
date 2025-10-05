# Bow - Stow on steriods

<small>(it's pronounced "bow" as in like, the archery thing)</small>

## Features

- Declaratively manage user-level packages from many different sources (Flatpak, AppImage, your system package manager, or just raw binaries!)
- Manage your dotfiles via symlinks
    - Drop in replacement for [GNU Stow](https://www.gnu.org/software/stow/) (this is a huge goal)
- Modes to either just install what you list or idempotent-ly manage your packages by removing any (user) packages that are installed but not listed in your config
- Super simple config through a single YAML file
- Asynchronous installation of packages (HUUUUUUGE todo...)
- Oh and it's written in rust :3

## Supported package providers

<small>(This is a checklist for now)</small>

- [ ] Flatpak (in progress)
- [ ] Binaries (in progress)
- [ ] AppImage
- [ ] System package management
- [ ] Snap maybe? Idk I've literally never used snaps
- [ ] Dotfiles (symlinking files to `$HOME`)
