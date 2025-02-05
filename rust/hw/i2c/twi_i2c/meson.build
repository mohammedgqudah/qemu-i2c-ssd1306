subproject('bilge-0.2-rs', required: true)
subproject('bilge-impl-0.2-rs', required: true)

bilge_dep = dependency('bilge-0.2-rs')
bilge_impl_dep = dependency('bilge-impl-0.2-rs')

_libtwi_i2c_rs = static_library(
  'twi_i2c',
  files('src/lib.rs'),
  override_options: ['rust_std=2021', 'build.rust_std=2021'],
  rust_abi: 'rust',
  dependencies: [
    bilge_dep,
    bilge_impl_dep,
    qemu_api,
    qemu_api_macros,
  ],
)

rust_devices_ss.add(when: 'CONFIG_TWI_I2C', if_true: [declare_dependency(
  link_whole: [_libtwi_i2c_rs],
  # Putting proc macro crates in `dependencies` is necessary for Meson to find
  # them when compiling the root per-target static rust lib.
  dependencies: [bilge_impl_dep, qemu_api_macros],
  variables: {'crate': 'twi_i2c'},
)])
