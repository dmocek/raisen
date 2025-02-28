# raisen 🏃‍

[![Build Status](https://travis-ci.org/Soft/run-or-raise.svg?branch=master)](https://travis-ci.org/Soft/run-or-raise)
[![Latest Version](https://img.shields.io/crates/v/run-or-raise.svg)](https://crates.io/crates/run-or-raise)
[![GitHub release](https://img.shields.io/github/release/Soft/run-or-raise.svg)](https://github.com/Soft/run-or-raise/releases)
[![dependency status](https://deps.rs/repo/github/soft/run-or-raise/status.svg)](https://deps.rs/repo/github/soft/run-or-raise)
[![AUR version](https://img.shields.io/aur/version/run-or-raise.svg)](https://aur.archlinux.org/packages/run-or-raise/)
[![License: GPL v3](https://img.shields.io/badge/License-GPL%20v3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

`raisen` is a utility for launching applications or focusing their windows
if they are already running. When invoked, `raisen` tries to find a window
that matches a specified criteria and focus it or, if no matching window is
found, execute a specified program.

This can be useful when combined with a tiling window manager such as
[i3](https://i3wm.org) or a general purpose keyboard shortcut manager such as
[xbindkeys](http://www.nongnu.org/xbindkeys/) that allow binding arbitrary
commands to keybindings. In such setup, one might use `raisen` to, for
example, launch or focus a web browser with a single key press.

`raisen` is designed to work with X11 based Linux systems.

## Installation

`raisen` can be installed using
[cargo](https://doc.rust-lang.org/cargo/index.html):

``` shell
cargo install raisen
```

Compiling and running `raisen` requires [libxcb](https://xcb.freedesktop.org)
library to be installed.

To get the latest development version of `raisen`, you can direct cargo to
install from the git repository:

``` shell
cargo install --git 'https://github.com/Soft/raisen.git'
```

Note that cargo will not install man pages. To install `raisen` along with
its manual invoke `make install` in the project directory. By default, the
installation script will place the files under `/usr/local/` hierarchy.

## Usage

``` text
raisen CONDITION PROGRAM [ARGS...]
```

When invoked, `raisen` matches existing windows against `CONDITION`. If a
matching window is found, it is focused. If none of the windows match the
criteria, `raisen` executes `PROGRAM` passing any `ARGS` to it as
arguments.

## Conditions

Conditions select windows based on their properties. In X11, each window can
have any number of properties associated with them. Examples of window
properties include *name* (typically what is visible in window's title bar),
*class* (an identifier that can be usually used to select windows of a
particular applications) and *role* (a representation of window's logical role,
eg. a web browser). The [xprop](https://www.x.org/releases/X11R7.5/doc/man/man1/xprop.1.html)
utility can be used to inspect windows and their properties.

The simplest possible window matching condition simply compares one of the
properties with a value:

``` shell
raisen 'name = "Spotify"' spotify
```

This would find and focus a window with the title “Spotify” or run the command
`spotify`.

Conditions support two comparison operators: `=` for exact equality comparison
with a string literal and `~` for comparing using a [regular
expression](https://en.wikipedia.org/wiki/Regular_expression).

Comparisons can be combined using logical operators: `&&` for logical *AND*,
`||` for logical *OR*, and `!` for logical *NOT*. Operators in matching
expressions are left-associative and `!` (not) binds stronger than `&&` (and)
which, in turn, binds stronger than `||` (or). Possible properties are `class`,
`name`, and `role`. Additionally, parentheses can be used to alter evaluation
order. Strings and regular expressions are written inside double quotes. If
multiple windows match the criteria, the first matching window is selected.

Bellow are some examples of how conditions can be used to select windows in
various ways:

``` shell
# Launch or focus emacs
raisen 'class = "Emacs"' emacs

# You can also use regular expressions for matching.
# Match windows with title ending with the string "Firefox"
raisen 'name ~ ".*Firefox$"' firefox

# You can combine multiple comparisons with logical operators.
# Match windows with the role "browser" that do not have the class "Chromium".
raisen 'role = "browser" && ! class = "Chromium"' firefox

# Even more complex conditions are possible.
# This is getting silly
raisen '! name ~ ".*\d+.*" || role = "browser" && ! class = "Emacs"' urxvt
```

## Integration with External Tools

`raisen` can be combined with just about any tool that allows executing
arbitrary commands in response to key events. Bellow are some hints about
configuring `raisen` to work with various applications:

### xbindkeys Keyboard Shortcut Manager

[xbindkeys](http://www.nongnu.org/xbindkeys/) is an application for executing
commands based on key events. `raisen` can be combined with it to only
launch applications if they are not already running. For example, to launch or
focus Firefox by pressing `Shift+Mod4+b`, one could use the following
`xbindkeys` configuration:

``` shell
"raisen 'role = \"browser\"' firefox"
	Shift+Mod4+b
```

### i3 Window Manager

[i3](https://i3wm.org) is a tiling window manager that, among other things,
supports binding arbitrary commands to arbitrary keys. To bind `raisen`
invocation to a key with i3, one might specify something like the following in
i3's configuration file:

``` shell
bindsym Mod4+Shift+b exec --no-startup-id \
	raisen 'role = "browser"' firefox
```

### KDE Custom Shortcuts

[KDE](https://www.kde.org) allows binding arbitrary commands to key presses
using [Custom Shortcuts manager](https://docs.kde.org/trunk5/en/kde-workspace/kcontrol/khotkeys/index.html#intro).
Through this graphical configuration utility, `raisen` can be used to
launch or focus applications.

### Desktop Entries

[Desktop Entries](https://developer.gnome.org/integration-guide/stable/desktop-files.html.en)
are used to define shortcuts that appear in application menus and launchers. In
addition to application name and icon they also define what commands should be
executed when an application is launched. `raisen` can be used as a part
of a desktop file to mandate that only a single instance of a particular
application should be started. For example, Spotify on Linux does not currently
enforce that only a single instance of the application can be launched, this is
annoying since having multiple audio players open is rarely what one wants.
Integrating `raisen` into a desktop file means replacing the `Exec` key
with a one that invokes `raisen` to check if the application is already
running:

``` desktop
[Desktop Entry]
Name=Spotify
Exec=raisen 'class = "Spotify"' spotify %U
...
```

