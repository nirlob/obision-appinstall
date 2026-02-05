project('libadwaita-demo-c',
  'c',
  version: '0.1.0',
  meson_version: '>= 0.59.0',
)

application_id = 'com.obision.example.C'

# Dependencies
gtk_dep = dependency('gtk4', version: '>= 4.10')
adw_dep = dependency('libadwaita-1', version: '>= 1.5')

# Source files
sources = files('src/main.c')

# Resources
gnome = import('gnome')
resources = gnome.compile_resources(
  'demo-resources',
  'data/demo.gresource.xml',
  source_dir: 'data',
  c_name: 'demo',
)

# Executable
executable(
  'example-c',
  sources + resources,
  dependencies: [gtk_dep, adw_dep],
  install: true,
)

# Install data files
subdir('data')
