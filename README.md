# RUST Backup

### Supported commands, while the GUI is closed and the daemon is running in background
- `CTRL + SHIFT + ALT + G` Shows up the GUI
- `CTRL + SHIFT + ALT + B` Shows up the GUI and execute an immediate backup

### Settings.dat file
| Setting name | Default value | Description |
|-|-|-|
| paths_to_backup | `Home Directory` | A list of paths of directories and files to backup, each separated by \n
| keep_daemon_alive | `true` | Allow the program to run in background for listening the Key combinations
| multi_thread | `true` | If enabled, spawn N_CPU_cores threads that backup N_CPU_cores paths simultaneously
| default_backup_path | `./backup/` | The path where to store the backup
