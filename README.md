# RUST Backup
A lightweight utility for emergency backups. Compatible with Linux and Windows systems.
Made with [RUST](https://www.rust-lang.org/) and [Slint UI Framework](https://slint.dev/)

# Installation
## Windows
### The following installation steps were tested on a Windows 10 Home Edition
- Download the zip package [here]()
- Open Run command box by pressing the Windows logo + R keys. In the Run command field, type `shell:startup` and then press Enter key to open Startup folder
- Extract the executable inside the Startup folder
- Now this utility will be automatically opened when the user logs in.

## Linux
### The following installation steps were tested on a Debian 11 with KDE Plasma
#### Install the following dependencies for Slint:
- Package config installation
  - Install: `sudo apt-get install pkg-config`
- X11 library:
  - Install: `sudo apt-get install libx11-dev libx11-xcb-dev`
  - Get installation path: `find /usr -name x11.pc`
  - Edit $PKG_CONFIG_PATH: `export PKG_CONFIG_PATH=`previous_output_path`:$PKG_CONFIG_PATH`
- xi development package
  - Install: `sudo apt-get install libxi-dev`
  - Get installation path: `find /usr -name xi.pc`
  - Edit $PKG_CONFIG_PATH: `export PKG_CONFIG_PATH=`previous_output_path`:$PKG_CONFIG_PATH`
- xtst development package
  - Install: `sudo apt-get install libxtst-dev`
  - Get installation path: `find /usr -name xi.pc`
  - Edit $PKG_CONFIG_PATH: `export PKG_CONFIG_PATH=`previous_output_path`:$PKG_CONFIG_PATH`

#### Installation steps
- Download the zip package [here]()
- Extract it inside a folder you like
- Edit your ~/.profile in your home directory and add the following content:
`if [ "$DISPLAY" ]; then
    /path/to/rust_backup &
fi`
- Now this utility will be automatically opened when the user logs in.

# Supported commands, while the GUI is closed and the daemon is running in background
- `CTRL + SHIFT + ALT + G` Shows up the GUI
- `CTRL + SHIFT + ALT + B` Shows up the GUI and execute an immediate backup

# Settings.dat file
| Setting name | Default value | Description |
|-|-|-|
| paths_to_backup | `Home Directory` | A list of paths of directories and files to backup, each separated by \n
| keep_daemon_alive | `true` | Allow the program to run in background for listening the Key combinations
| multi_thread | `true` | If enabled, spawn N_CPU_cores threads that backup N_CPU_cores paths simultaneously
| default_backup_path | `./backup/` | The path where to store the backup
