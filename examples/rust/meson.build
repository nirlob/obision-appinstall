project('libadwaita-example',
  'rust',
  version: '0.1.0',
  meson_version: '>= 0.59.0',
)

application_id = 'com.obision.example.Rust'

# Dependencias
dependency('gtk4', version: '>= 4.10')
dependency('libadwaita-1', version: '>= 1.5')

# Compilar con cargo
cargo_script = find_program('build-aux/cargo.sh')
# Ejecutable
app_bin = custom_target(
  'obision-example',
  build_by_default: true,
  build_always_stale: true,
  output: 'example-rust',
  console: true,
  install: true,
  install_dir: get_option('bindir'),
  command: [
    cargo_script,
    '@SOURCE_ROOT@',
    '@BUILD_ROOT@',
    '@OUTPUT@',
    get_option('buildtype'),
    'example-rust',
  ]
)

# Instalar archivos de datos
subdir('data')
