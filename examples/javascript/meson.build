project('libadwaita-demo-js',
  version: '0.1.0',
  meson_version: '>= 0.59.0',
)

application_id = 'com.obision.example.JavaScript'

# Check for GJS
gjs = find_program('gjs', required: true)

# Dependencies
dependency('gtk4', version: '>= 4.10')
dependency('libadwaita-1', version: '>= 1.5')

# Install GJS script as executable
install_data(
  'src/main.js',
  install_dir: get_option ('bindir'),
  rename: 'example-javascript',
  install_mode: 'rwxr-xr-x',
)

# Install data files
subdir('data')
