# Examples - Multi-Language Libadwaita Demos

Este directorio contiene aplicaciones de ejemplo en múltiples lenguajes para probar el sistema de empaquetado `.lis`.

## Ejemplos Disponibles

### 🦀 Rust (`rust/`)
Aplicación Libadwaita demo escrita en Rust con GTK4.
- **App ID**: `com.obision.example.Rust`
- **Build**: `cd rust && meson setup builddir && meson compile -C builddir`
- **Ejecutable**: `example-rust`
- **Run**: `./builddir/example-rust` o `cargo run`

### 🔵 C (`c/`)
Aplicación Libadwaita demo escrita en C con GTK4.
- **App ID**: `com.obision.example.C`
- **Build**: `cd c && meson setup builddir && meson compile -C builddir`
- **Ejecutable**: `example-c`
- **Run**: `./builddir/example-c`

### ➕ C++ (`cpp/`)
Aplicación Libadwaita demo escrita en C++ usando GTK4 C API.
- **App ID**: `com.obision.example.Cpp`
- **Build**: `cd cpp && meson setup builddir && meson compile -C builddir`
- **Ejecutable**: `example-cpp`
- **Run**: `./builddir/example-cpp`

### 🐍 Python (`python/`)
Aplicación Libadwaita demo escrita en Python usando PyGObject.
- **App ID**: `com.obision.example.Python`
- **Build**: `cd python && meson setup builddir`
- **Ejecutable**: `example-python`
- **Run**: `cd python/src && python3 main.py`
- **Requisitos**: `python3`, `python3-gi`, `gir1.2-gtk-4.0`, `gir1.2-adw-1`

### 📜 JavaScript (`javascript/`)
Aplicación Libadwaita demo escrita en JavaScript usando GJS.
- **App ID**: `com.obision.example.JavaScript`
- **Build**: `cd javascript && meson setup builddir`
- **Ejecutable**: `example-javascript`
- **Run**: `cd javascript && gjs src/main.js`
- **Requisitos**: `gjs`

## Ejecutar Todos los Ejemplos Simultáneamente

```bash
# Desde el directorio examples/
cd rust && ./builddir/example-rust &
cd ../c && ./builddir/example-c &
cd ../cpp && ./builddir/example-cpp &
cd ../python/src && python3 main.py &
cd ../../javascript && gjs src/main.js &
```

## Propósito

Estos ejemplos sirven para:
1. **Probar el empaquetador** (`obision-builder`) con diferentes lenguajes
2. **Demostrar compatibilidad** del formato `.lis` con múltiples tecnologías
3. **Referencias de implementación** para aplicaciones GNOME con Libadwaita
4. **Demostrar uso de archivos .ui** para diseño de interfaz

## Instalación

Cada ejemplo se puede instalar de forma independiente:

```bash
cd <ejemplo>
meson setup builddir
meson compile -C builddir  # Solo para lenguajes compilados (Rust, C, C++)
sudo meson install -C builddir
```

Después de instalar, cada aplicación aparecerá en el menú de aplicaciones de GNOME.

## Empaquetado con .lis

Una vez que `obision-builder` esté completo, podrás generar paquetes `.lis` para cada ejemplo:

```bash
# Usar builder para crear paquete
obision-builder

# Seleccionar directorio del ejemplo (ej: rust/)
# Generar paquete -> example-rust.lis

# Instalar con installer
obision-installer
```

## Estructura de Cada Ejemplo

Todos siguen la misma estructura estándar:

```
<lenguaje>/
├── meson.build                      # Configuración de build
├── src/
│   └── main.<ext>                   # Código fuente
├── data/
│   ├── meson.build
│   ├── window.ui                    # Interfaz GTK (UI file)
│   ├── com.obision.example.<Lang>.desktop
│   ├── com.obision.example.<Lang>.metainfo.xml
│   └── icons/scalable/
│       └── com.obision.example.<Lang>.svg
└── README.md (opcional)
```

## Características Comunes

Todos los ejemplos demuestran:
- ✅ **Archivo .ui** para diseño de interfaz GTK
- ✅ **Libadwaita widgets** modernos
- ✅ **Header bar** con `AdwHeaderBar` y título personalizado
- ✅ **Responsive layout** con `AdwClamp`
- ✅ **AdwStatusPage** con emoji distintivo por lenguaje
- ✅ **AdwPreferencesGroup** con filas interactivas
- ✅ **Botones** con estilos pill y suggested-action
- ✅ **Dark mode switch** funcional (en ejemplos compilados)
- ✅ **Integración completa** con GNOME
- ✅ **App IDs únicos** bajo namespace `com.obision.example.*`
- ✅ **Nombres de ejecutables** estandarizados: `example-*`

## Funcionalidad de los Ejemplos

Cada ejemplo incluye:
- **Botón Principal**: Al hacer clic, cambia su etiqueta a "¡Clickeado!"
- **Dark Mode Switch**: Alterna entre tema claro y oscuro (Rust tiene esta implementación)
- **UI consistente**: Todas las ventanas usan el mismo diseño `.ui` con personalizaciones por lenguaje

## Dependencias de Build

### Comunes a todos:
- `meson >= 0.59.0`
- `gtk4 >= 4.10`
- `libadwaita-1 >= 1.5`

### Específicas por lenguaje:
- **Rust**: `rustc`, `cargo`
- **C/C++**: `gcc` o `clang`
- **Python**: `python3`, `python3-gi`
- **JavaScript**: `gjs`
