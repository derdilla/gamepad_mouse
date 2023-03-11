## Setup
The controller's name needs to contain the word "gamepad" or "controller". If you need to change mouse speed, add a word unique to your controllers name or change other parameters, you can to recompile it with the constants on top of the `main.rs` file set.

## Installation
Simply run the executable from the latest release on any Linux distro with X11. (Sorry windows and Mac users)

### Runtime dependencies

You may have to install `libxdo-dev`. For example, on Debian-based distros:

```Bash
apt-get install libxdo-dev
```

On Arch:

```Bash
pacman -S xdotool
```

On Fedora:

```Bash
dnf install libX11-devel libxdo-devel
```

On Gentoo:

```Bash
emerge -a xdotool
```

## Bindings
![Controller_Bindings](controller_bindings.png)
