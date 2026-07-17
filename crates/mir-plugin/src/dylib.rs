//! Runtime loading of Rust plugins from cdylibs.
//!
//! A plugin crate sets `crate-type = ["cdylib"]`, depends on `mir-plugin`,
//! and exports its constructor with [`crate::export_plugin!`]. The resulting
//! library is referenced from `mir.xml`:
//!
//! ```xml
//! <plugins>
//!     <rustPlugin path="plugins/libmy_plugin.dylib"/>
//! </plugins>
//! ```

use std::path::Path;

use crate::{MirPlugin, PluginDeclaration, MIR_PLUGIN_API_VERSION};

#[derive(Debug, thiserror::Error)]
pub enum DylibError {
    #[error("cannot load plugin library {path}: {source}")]
    Load {
        path: String,
        source: libloading::Error,
    },
    #[error("{path} exports no MIR_PLUGIN_DECLARATION symbol — not a mir plugin (did you use mir_plugin::export_plugin!?)")]
    MissingDeclaration { path: String },
    #[error("{path} was built against mir plugin API v{found}, this binary requires v{expected}")]
    ApiVersionMismatch {
        path: String,
        found: u32,
        expected: u32,
    },
}

/// Load a Rust plugin from a cdylib. The library is intentionally leaked:
/// plugins live for the lifetime of the process, and unloading code that
/// still has vtable pointers registered is never safe.
///
/// # Safety contract
/// The dylib must be built with the same Rust toolchain and the same
/// `mir-plugin` crate version as this binary. `api_version` is checked, but
/// Rust has no stable ABI — a mismatched toolchain is undefined behavior.
pub fn load(path: &Path) -> Result<Box<dyn MirPlugin>, DylibError> {
    let display = path.display().to_string();
    // SAFETY: loading foreign code is inherently trusted; see contract above.
    let lib = unsafe { libloading::Library::new(path) }.map_err(|source| DylibError::Load {
        path: display.clone(),
        source,
    })?;
    let decl = unsafe { lib.get::<*const PluginDeclaration>(b"MIR_PLUGIN_DECLARATION\0") }
        .map_err(|_| DylibError::MissingDeclaration {
            path: display.clone(),
        })?;
    // SAFETY: the symbol is a static exported by export_plugin!.
    let decl: &PluginDeclaration = unsafe { &**decl };
    if decl.api_version != MIR_PLUGIN_API_VERSION {
        return Err(DylibError::ApiVersionMismatch {
            path: display,
            found: decl.api_version,
            expected: MIR_PLUGIN_API_VERSION,
        });
    }
    let plugin = (decl.create)();
    std::mem::forget(lib);
    Ok(plugin)
}
