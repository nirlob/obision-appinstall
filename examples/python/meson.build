project('libadwaita-demo-python',
  version: '0.1.0',
  meson_version: '>= 0.59.0',
)

application_id = 'com.obision.example.Python'

# Check for Python
python = import('python').find_installation('python3')

# Dependencies (checked at build time, but runtime is what matters for Python)
dependency('gtk4', version: '>= 4.10')
dependency('libadwaita-1', version: '>= 1.5')

# Install Python script as executable
install_data(
  'src/main.py',
  install_dir: get_option ('bindir'),
  rename: 'example-python',
  install_mode: 'rwxr-xr-x',
)

# Install data files
subdir('data')
