project('obision-appinstall-installer',
  'rust',
  version: '0.1.0',
  meson_version: '>= 0.59.0',
)

app_id = 'com.obision.appinstall.Installer'

# Dependencias
dependency('gtk4', version: '>= 4.10')
dependency('libadwaita-1', version: '>= 1.5')

# Compilar con cargo
cargo_script = find_program('build-aux/cargo.sh')
cargo_release = custom_target(
  'cargo-build',
  build_by_default: true,
  build_always_stale: true,
  output: 'obision-appinstall-installer',
  console: true,
  install: true,
  install_dir: get_option('bindir'),
  command: [
    cargo_script,
    meson.project_build_root(),
    meson.project_source_root(),
    '@OUTPUT@',
    get_option('buildtype'),
    'obision-appinstall-installer',
  ]
)

# Instalar archivos de datos
subdir('data')
