_libssd1306_rs = static_library(
  'ssd1306',
  files('src/lib.rs'),
  override_options: ['rust_std=2021', 'build.rust_std=2021'],
  rust_abi: 'rust',
  dependencies: [
    qemu_api,
    qemu_api_macros,
  ],
)

rust_devices_ss.add(when: 'CONFIG_SSD1306', if_true: [declare_dependency(
  link_whole: [_libssd1306_rs],
  dependencies: [qemu_api_macros],
  variables: {'crate': 'ssd1306'},
)])
