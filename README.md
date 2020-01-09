# Poki Launcher

**Work in progress**

Poki Launcher is an application launcher for Linux.  It tracks app usage then ranks apps based on frecency and fuzzy search.
It's written in Rust and QML.

![Launcher Image](./media/launcher.png)

# How to Install

## OpenSUSE & Fedora

Get package from OBS: https://software.opensuse.org//download.html?project=home%3Azethra&package=poki-launcher


## From source

Install the Qt5 base and declarative development packages for your distro.
Ex.
Fedora `dnf install qt5-qtbase-devel qt5-qtdeclarative-devel`
OpenSUSE `libqt5-qtbase-devel libqt5-qtdeclarative-devel`

Also install `wmctrl`

```
git clone https://github.com/zethra/poki-launcher.git && cd poki-launcher
cargo install --path poki-launcher
```

# Configuration

To change any setting in the app copy the example config file installed
at `/usr/share/doc/packages/poki-launcher/poki-launcher.hjson` if installed
from the package other grab it from `https://raw.githubusercontent.com/zethra/poki-launcher/master/poki-launcher.hjson`.
Copy this file to `~/.config/poki-launcher/poki-launcher.hjson`.

## Config options

- `app_paths`

A list of paths to search for desktop files in.  Defaults to just `/usr/share/applications/`

- `term_cmd`

Override the command used to launch terminal apps.  Defaults to `$TERM -e`


# Trouble shotting

**Q** An app isn't in the list

**A** If the app's desktop file is in a directory that's
not in the `app_paths` list in the config file they won't
show up.  If the app was installed with flatpak or snap
uncomment the lines for those in the example config file.

## Otherwise

If you have any issues with the app or a question send me and email
detailing the issue at [benaagoldberg@gmail.com](mailto:benaagoldberg@gmail.com) with "POKI LAUNCHER ISSUE"
in the subject line or create an issue on my github page.