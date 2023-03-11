## Setup
The controllers name needs to contain the word "gamepad". If you need to change mouse speed or other parameters, you can to recompile it with the constants set.

## Installation
Simply run the executable from the latest release on Linux with X11. (Sorry windows and mac users)

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
