project('obision-appinstall-builder',
  version: '0.1.0',
  meson_version: '>= 0.59.0',
)

app_id = 'com.obision.appinstall.Builder'

dependency('glib-2.0', version: '>= 2.66')
dependency('gio-2.0', version: '>= 2.66')
dependency('gtk4', version: '>= 4.0.0')

cargo = find_program('cargo')
cargo_script = find_program('build-aux/cargo.sh')

custom_target(
  'obision-appinstall-builder',
  build_by_default: true,
  build_always_stale: true,
  output: 'obision-appinstall-builder',
  console: true,
  install: true,
  install_dir: get_option('bindir'),
  command: [
    cargo_script,
    meson.project_build_root(),
    meson.project_source_root(),
    '@OUTPUT@',
    get_option('buildtype'),
    'obision-appinstall-builder',
  ]
)

# Instalar archivos de datos
subdir('data')
