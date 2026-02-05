# UI file
install_data(
  'ui/window.ui',
  install_dir: get_option('datadir') / app_id / 'ui',
)

# Desktop file
desktop_file = app_id + '.desktop'
install_data(
  desktop_file,
  install_dir: get_option('datadir') / 'applications',
)

# Metainfo file
metainfo_file = app_id + '.metainfo.xml'
install_data(
  metainfo_file,
  install_dir: get_option('datadir') / 'metainfo',
)

# Icon
icon_sizes = ['scalable']
foreach size : icon_sizes
  install_data(
    'icons' / size / (app_id + '.svg'),
    install_dir: get_option('datadir') / 'icons' / 'hicolor' / size / 'apps',
  )
endforeach
