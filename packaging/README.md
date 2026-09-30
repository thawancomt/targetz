# Windows Packaging & Installer Guide for Targetz

This directory contains the installer definitions and build automation scripts to produce **`Targetz-Setup-x86_64.exe`** (an installable `.exe` wizard).

---

## 1. Quick Build on Windows (Recommended)

### Prerequisites
- [Rust](https://rustup.rs/) (with `x86_64-pc-windows-msvc` target installed)
- [Inno Setup 6](https://jrsoftware.org/isdl.php) (can be installed via `winget install JRSoftware.InnoSetup`) or [NSIS](https://nsis.sourceforge.io/) (`winget install NSIS.NSIS`)

### Build
Run from the root of the project:
```cmd
packaging\build-installer.bat
```
or manually:
```cmd
cargo build --release --bin targetz --target x86_64-pc-windows-msvc
iscc packaging\installer.iss
```
The installable setup file is produced in `dist/Targetz-Setup-x86_64.exe`.

---

## 2. Cross-Compiling from Linux

### Prerequisites
1. **Rust target**:
   ```bash
   rustup target add x86_64-pc-windows-msvc
   ```
2. **`cargo-xwin`** (allows compiling MSVC targets on Linux):
   ```bash
   cargo install --locked cargo-xwin
   ```
3. **NSIS** (to package the installer):
   ```bash
   # On Fedora / RHEL
   sudo dnf install mingw32-nsis
   # On Ubuntu / Debian
   sudo apt install nsis
   ```

### Build
```bash
# 1. Compile the Windows binary
cargo xwin build --release --bin targetz --target x86_64-pc-windows-msvc

# 2. Package into the setup wizard
mkdir -p dist
makensis packaging/installer.nsi
```
The output will be generated at `dist/Targetz-Setup-x86_64.exe`.

---

## 3. Installer Features

- **No Administrator Privileges Required**: Installs to `%LOCALAPPDATA%\Programs\Targetz`.
- **Shortcuts**: Automatically adds Start Menu and Desktop shortcuts.
- **Uninstaller**: Adds clean uninstaller in Windows "Installed apps" / "Add or Remove Programs".
- **Self-Contained**: Statically bundles SQLite (`sqlx` bundled), icons, themes, and migrations.
