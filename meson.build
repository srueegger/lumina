project(
  'lumina',
  'rust',
  version: '0.1.0',
  meson_version: '>= 0.63',
  license: 'GPL-2.0-only',
)

gnome = import('gnome')
i18n = import('i18n')

base_id = 'me.rueegger.Lumina'

dependency('glib-2.0', version: '>= 2.76')
dependency('gio-2.0', version: '>= 2.76')
dependency('gtk4', version: '>= 4.12')
dependency('libadwaita-1', version: '>= 1.5')

profile = get_option('profile')
if profile == 'development'
  app_id = '@0@.Devel'.format(base_id)
  vcs_tag = run_command('git', 'rev-parse', '--short', 'HEAD', check: false).stdout().strip()
  if vcs_tag == ''
    version_suffix = '-devel'
  else
    version_suffix = '-@0@'.format(vcs_tag)
  endif
else
  app_id = base_id
  version_suffix = ''
endif

prefix = get_option('prefix')
bindir = prefix / get_option('bindir')
localedir = prefix / get_option('localedir')
datadir = prefix / get_option('datadir')
pkgdatadir = datadir / meson.project_name()

subdir('data')
subdir('po')
subdir('src')

gnome.post_install(
  gtk_update_icon_cache: true,
  glib_compile_schemas: true,
  update_desktop_database: true,
)
