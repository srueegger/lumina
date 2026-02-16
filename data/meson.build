desktop_conf = configuration_data()
desktop_conf.set('APP_ID', app_id)
desktop_conf.set('BINARY', meson.project_name())

desktop_file = i18n.merge_file(
  input: configure_file(
    input: '@0@.desktop.in.in'.format(base_id),
    output: '@0@.desktop.in'.format(app_id),
    configuration: desktop_conf,
  ),
  output: '@0@.desktop'.format(app_id),
  type: 'desktop',
  po_dir: meson.project_source_root() / 'po',
  install: true,
  install_dir: datadir / 'applications',
)

desktop_utils = find_program('desktop-file-validate', required: false)
if desktop_utils.found()
  test('Validate desktop file', desktop_utils, args: [desktop_file])
endif

metainfo_conf = configuration_data()
metainfo_conf.set('APP_ID', app_id)

metainfo_file = i18n.merge_file(
  input: configure_file(
    input: '@0@.metainfo.xml.in'.format(base_id),
    output: '@0@.metainfo.xml.in'.format(app_id),
    configuration: metainfo_conf,
  ),
  output: '@0@.metainfo.xml'.format(app_id),
  type: 'xml',
  po_dir: meson.project_source_root() / 'po',
  install: true,
  install_dir: datadir / 'metainfo',
)

appstreamcli = find_program('appstreamcli', required: false)
if appstreamcli.found()
  test('Validate metainfo file', appstreamcli,
    args: ['validate', '--no-net', '--explain', metainfo_file],
  )
endif

schema_conf = configuration_data()
schema_conf.set('APP_ID', app_id)

configure_file(
  input: '@0@.gschema.xml.in'.format(base_id),
  output: '@0@.gschema.xml'.format(app_id),
  configuration: schema_conf,
  install: true,
  install_dir: datadir / 'glib-2.0' / 'schemas',
)

compile_schemas = find_program('glib-compile-schemas', required: false)
if compile_schemas.found()
  test('Validate GSettings schemas', compile_schemas,
    args: ['--strict', '--dry-run', meson.current_build_dir()],
  )
endif

subdir('icons')
