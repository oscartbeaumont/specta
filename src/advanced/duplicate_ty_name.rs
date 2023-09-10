use std::{borrow::Cow, collections::HashMap};

use crate::{ImplLocation, TypeMap};

/// post process the type map to detect duplicate type names
pub fn detect_duplicate_type_names(
    type_map: &TypeMap,
) -> Vec<(Cow<'static, str>, ImplLocation, ImplLocation)> {
    let mut errors = Vec::new();

    let mut map = HashMap::with_capacity(type_map.len());
    for (sid, dt) in type_map {
        match dt {
            Some(dt) => {
                if let Some(ext) = &dt.ext {
                    if let Some((existing_sid, existing_impl_location)) =
                        map.insert(dt.name.clone(), (sid, ext.impl_location))
                    {
                        if existing_sid != sid {
                            errors.push((
                                dt.name.clone(),
                                ext.impl_location,
                                existing_impl_location,
                            ));
                        }
                    }
                }
            }
            None => unreachable!(),
        }
    }

    errors
}
