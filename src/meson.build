cargo = find_program('cargo')
cargo_build_script = find_program(
  meson.project_source_root() / 'build-aux' / 'cargo-build.sh'
)

if profile == 'development'
  rust_target = 'debug'
else
  rust_target = 'release'
endif

cargo_env = environment()
cargo_env.set('MESON_SOURCE_ROOT', meson.project_source_root())
cargo_env.set('MESON_BUILD_ROOT', meson.project_build_root())
cargo_env.set('APP_ID', app_id)
cargo_env.set('LOCALEDIR', localedir)
cargo_env.set('PKGDATADIR', pkgdatadir)
cargo_env.set('VERSION', meson.project_version() + version_suffix)

cargo_build = custom_target(
  'cargo-build',
  build_by_default: true,
  build_always_stale: true,
  output: meson.project_name(),
  console: true,
  install: true,
  install_dir: bindir,
  env: cargo_env,
  command: [
    cargo_build_script,
    rust_target,
    meson.project_name(),
    '@OUTPUT@',
  ],
)
