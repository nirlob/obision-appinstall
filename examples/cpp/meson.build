project('libadwaita-demo-cpp',
  'cpp',
  version: '0.1.0',
  meson_version: '>= 0.59.0',
  default_options: ['cpp_std=c++17']
)

application_id = 'com.obision.example.Cpp'

# Dependencies (using C API from C++)
gtk_dep = dependency('gtk4', version: '>= 4.10')
adw_dep = dependency('libadwaita-1', version: '>= 1.5')
gio_dep = dependency('gio-2.0')

# Source files
sources = files('src/main.cpp')

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
  'example-cpp',
  sources + resources,
  dependencies: [gtk_dep, adw_dep, gio_dep],
  install: true,
)

# Install data files
subdir('data')
