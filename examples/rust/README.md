# Libadwaita Example - Rust

Este es un proyecto de ejemplo que demuestra el uso de libadwaita con Rust.

## Características

- ✨ Interfaz moderna usando Libadwaita
- 🎨 Componentes nativos de GNOME
- 🗂️ **Interfaz definida con archivo .ui de GTK** (separación UI/lógica)
- ⚡ Header bar personalizada
- 📱 Diseño responsivo con Clamp
- 🔘 Botones con estilos Adwaita (pill, suggested-action, destructive-action)
- ⚙️ Preference rows con switches
- 📄 Status page de bienvenida
- 🌓 Cambio dinámico entre modo claro/oscuro

## Requisitos

Necesitas tener instalado en tu sistema:

- Rust (1.70 o superior)
- **pkg-config** (requerido para compilar)
- GTK 4 (versión 4.10 o superior)
- Libadwaita (versión 1.5 o superior)

### Instalación de dependencias en sistemas basados en Debian/Ubuntu

```bash
sudo apt install pkg-config libgtk-4-dev libadwaita-1-dev build-essential
```

### Instalación de dependencias en Fedora

```bash
sudo dnf install pkg-config gtk4-devel libadwaita-devel gcc
```

### Instalación de dependencias en Arch Linux

```bash
sudo pacman -S pkgconf gtk4 libadwaita base-devel
```

## Compilación y ejecución

### Método 1: Compilación directa con Cargo (desarrollo rápido)

Para compilar y ejecutar el proyecto:

```bash
cargo run
```

Para compilar en modo release:

```bash
cargo build --release
```

El ejecutable estará en `target/release/libadwaita-example`

### Método 2: Instalación con Meson (integración completa con GNOME)

Este método instala la aplicación en tu sistema con soporte completo para GNOME, incluyendo:
- Entrada en el menú de aplicaciones
- Ícono de la aplicación
- Metadatos para GNOME Software

**Requisito adicional:**
```bash
sudo apt install meson ninja-build  # Debian/Ubuntu
sudo dnf install meson ninja-build  # Fedora
sudo pacman -S meson ninja          # Arch Linux
```

**Pasos para instalar:**

```bash
# Configurar el proyecto
meson setup builddir

# Compilar
meson compile -C builddir

# Instalar (requiere permisos de root)
sudo meson install -C builddir
```

Después de instalar, la aplicación aparecerá en el menú de aplicaciones de GNOME y podrás ejecutarla buscando "Libadwaita Demo".

**Para desinstalar:**
```bash
sudo ninja uninstall -C builddir
```

## Estructura del proyecto

```
example/
├── Cargo.toml                    # Dependencias y configuración del proyecto
├── meson.build                   # Sistema de compilación de Meson
├── build-aux/
│   └── cargo.sh                 # Script de integración Cargo-Meson
├── src/
│   └── main.rs                  # Código principal de la aplicación
├── data/
│   ├── window.ui                # Definición de la interfaz en formato GTK UI
│   ├── com.example.LibadwaitaDemo.desktop       # Archivo .desktop
│   ├── com.example.LibadwaitaDemo.metainfo.xml  # Metadatos AppStream
│   ├── meson.build              # Configuración de instalación de datos
│   └── icons/
│       └── scalable/
│           └── com.example.LibadwaitaDemo.svg   # Ícono de la aplicación
└── README.md                     # Este archivo
```

## Características del código

### Arquitectura

Este ejemplo demuestra las **mejores prácticas** de desarrollo GTK/Libadwaita en Rust:

1. **Separación UI/Lógica**: La interfaz está definida en `data/window.ui` (formato XML), mientras que la lógica de la aplicación está en `src/main.rs`
2. **Carga dinámica de UI**: Uso de `gtk4::Builder` para cargar la interfaz desde el archivo `.ui` usando `include_str!`
3. **Gestión de widgets**: Obtención de referencias a widgets específicos del archivo `.ui` por ID
4. **Conexión de señales**: Eventos como clicks y cambios de estado se conectan programáticamente desde Rust

### Componentes demostrados

1. **ApplicationWindow**: Ventana principal de la aplicación
2. **HeaderBar**: Barra de título con estilo Adwaita
3. **StatusPage**: Página de estado con ícono y descripción
4. **PreferencesGroup**: Grupo de preferencias
5. **ActionRow**: Filas de acción con switches
6. **Clamp**: Contenedor para diseño responsivo
7. **Botones estilizados**: Con clases CSS de Adwaita
8. **StyleManager**: Control dinámico del tema claro/oscuro

### Estilos CSS disponibles

- `suggested-action`: Botón de acción principal (azul)
- `destructive-action`: Botón de acción destructiva (rojo)
- `pill`: Bordes redondeados tipo píldora

### Ventajas del enfoque con archivos .ui

- ✅ **Mejor separación de responsabilidades**: UI separada de la lógica
- ✅ **Más fácil de editar**: La interfaz se puede modificar sin recompilar
- ✅ **Uso de herramientas visuales**: Compatible con Glade/Cambalache para diseño visual
- ✅ **Estándar de GNOME**: Siguiendo las convenciones de la plataforma GNOME
- ✅ **Más limpio**: Menos código repetitivo en Rust

## Integración con GNOME

El proyecto incluye soporte completo para integración con el escritorio GNOME:

### Archivo .desktop
El archivo `data/com.example.LibadwaitaDemo.desktop` permite que la aplicación aparezca en el menú de aplicaciones de GNOME con su nombre, descripción e ícono.

### Metainfo AppStream
El archivo `data/com.example.LibadwaitaDemo.metainfo.xml` proporciona metadatos detallados sobre la aplicación para GNOME Software y otros centros de software, incluyendo:
- Descripción completa
- Capturas de pantalla (configurables)
- Información de versiones
- Categorización

### Ícono de la aplicación
El ícono SVG en `data/icons/scalable/` sigue las directrices de diseño de GNOME con:
- Gradiente azul-morado característico de aplicaciones modernas
- Diseño de capas superpuestas representando componentes
- Formato escalable para diferentes tamaños

### Sistema de compilación Meson
Meson es el sistema de compilación estándar para aplicaciones GNOME. El archivo `meson.build` configura:
- Integración con Cargo para compilar el código Rust
- Instalación automática de archivos .desktop, metainfo e íconos
- Actualización de cachés del sistema

## Personalización

Puedes personalizar la aplicación modificando:

- El `application_id` en `main.rs`
- Los colores y estilos usando clases CSS de GTK/Adwaita
- Agregar más componentes de libadwaita como Toast, Banner, etc.

## Recursos adicionales

- [Documentación de gtk4-rs](https://gtk-rs.org/gtk4-rs/)
- [Documentación de libadwaita](https://gnome.pages.gitlab.gnome.org/libadwaita/)
- [Human Interface Guidelines de GNOME](https://developer.gnome.org/hig/)

## Licencia

Este ejemplo está en el dominio público. Úsalo como quieras.
