//! Internal types which power rspc. The module provides no guarantee of compatibility between updates, so you should be careful rely on types from it.
//!
//! WARNING: Anything in this module does not follow semantic versioning as it's considered an implementation detail.
//!

mod async_map;
pub mod jsonrpc;
mod jsonrpc_exec;
mod middleware;
mod procedure_builder;
mod procedure_store;

pub use async_map::*;
pub use middleware::*;
pub use procedure_builder::*;
pub use procedure_store::*;

#[cfg(not(feature = "unstable"))]
pub use specta;

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Write, path::PathBuf};

    use specta::{ts::export_datatype, DefOpts, Type, TypeDefs};

    macro_rules! collect_datatypes {
        ($( $i:path ),* $(,)? ) => {{
            use specta::DataType;

            let mut tys = TypeDefs::default();

            $({
                let def = <$i as Type>::definition(DefOpts {
                    parent_inline: true,
                    type_map: &mut tys,
                });

                if let Ok(def) = def {
                    if let DataType::Named(n) = def {
                        if let Some(sid) = n.sid {
                            tys.insert(sid, Some(n));
                        }
                    }
                }
            })*
            tys
        }};
    }

    // rspc has internal types that are shared between the frontend and backend. We use Specta directly to share these to avoid a whole class of bugs within the library itself.
    #[test]
    fn export_internal_types() {
        let mut file = File::create(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("./packages/client/src/bindings.ts"),
        )
        .unwrap();

        file.write_all(
            b"// DO NOT MODIFY. This file was generated by Specta and is used to keep rspc internally type safe.\n// Checkout the unit test 'export_internal_types' to see where this files comes from!",
        )
        .unwrap();

        let tys = collect_datatypes! {
            super::ProcedureDataType,
            // crate::Procedures, // TODO
            super::jsonrpc::Request,
            // super::jsonrpc::Response, // TODO
        };

        for (_, ty) in tys.into_iter().filter_map(|(sid, v)| v.map(|v| (sid, v))) {
            file.write_all(b"\n\n").unwrap();
            file.write_all(
                export_datatype(&Default::default(), &ty)
                    .unwrap()
                    .as_bytes(),
            )
            .unwrap();
        }
    }
}
