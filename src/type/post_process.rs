use std::collections::HashMap;

use crate::{ImplLocation, TypeDefs};

/// post process the type map to detect duplicate type names
#[doc(hidden)]
pub fn detect_duplicate_type_names(
    type_map: &TypeDefs,
) -> Vec<(&'static str, Option<ImplLocation>, Option<ImplLocation>)> {
    let mut errors = Vec::new();

    let mut map = HashMap::with_capacity(type_map.len());
    for (sid, dt) in type_map {
        match dt {
            Some(dt) => {
                if let Some((existing_sid, existing_impl_location)) =
                    map.insert(dt.name, (sid, dt.impl_location))
                {
                    if existing_sid != sid {
                        errors.push((dt.name, dt.impl_location, existing_impl_location));
                    }
                }
            }
            None => unreachable!(),
        }
    }

    errors
}
