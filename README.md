# Obision AppInstall

Obision AppInstall is the official package management system for the Obision operating system. It provides a complete ecosystem for creating, distributing, and installing applications using the `.lis` package format.

## Project Structure

The repository is organized into four independent components:

*   **`liblis/`**: The core shared library (`liblis.so`). It handles package metadata parsing, dependency resolution, and the low-level logic for the `.lis` format.
*   **`builder/`**: A graphical application (GTK4/Libadwaita) for developers. It allows creating new projects, configuring installer screens, and generating `.lis` packages.
*   **`installer/`**: The end-user graphical installer. It reads `.lis` packages and guides the user through the installation process (license agreement, path selection, etc.).
*   **`examples/`**: A collection of example applications in various languages (Rust, C, C++, Python, JavaScript) configured to be packaged with Obision AppInstall.

## Build Instructions

This project uses **Meson** as the build system. Since the components are independent but dependent on `liblis`, they must be built in a specific order.

### Prerequisites

*   **Meson** (>= 0.60.0)
*   **Ninja**
*   **Rust** (Cargo) and **Rustc**
*   **GTK4** and **Libadwaita** development headers
*   `pkg-config`

### 1. Build and Install `liblis` (Critical)

`liblis` must be installed first so that `builder` and `installer` can link against it.

```bash
cd liblis
meson setup builddir
meson compile -C builddir
sudo meson install -C builddir
```

Verify installation:
```bash
pkg-config --cflags --libs liblis
```

### 2. Build `builder`

Once `liblis` is installed:

```bash
cd builder
meson setup builddir
meson compile -C builddir
# Run directly:
./builddir/src/obision-appinstall-builder
# Or install:
sudo meson install -C builddir
```

### 3. Build `installer`

```bash
cd installer
meson setup builddir
meson compile -C builddir
# Run directly:
./builddir/src/obision-appinstall-installer
# Or install:
sudo meson install -C builddir
```

## Usage

### Creating a Package (Builder)

1.  Launch **Obision Builder**.
2.  Click **New Project**.
3.  Fill in the project details (Name, Version, Author).
4.  In the **Files** section, add the binaries and assets you want to package.
5.  In **Installer Screens**, toggle which steps the user will see (e.g., Welcome, License, Destination).
6.  Go to **Build Package** and click **Build**.

### Installing a Package

1.  Double-click any `.lis` file (if associated).
2.  Or run the installer with the package path:
    ```bash
    obision-installer myapp.lis
    ```
3.  Follow the wizard steps to install the application.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
