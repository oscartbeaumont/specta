macro_rules! impl_primitives {
    ($($i:ident)+) => {$(
        impl Type for $i {
            const MODULE_PATH: &'static str = module_path!();

            fn inline(_: DefOpts, _: &[DataType]) -> Result<DataType, ExportError> {
                Ok(DataType::Primitive(datatype::PrimitiveType::$i))
            }
        }
    )+};
}

macro_rules! impl_tuple {
    ( impl $i:ident ) => {
        impl_tuple!(impl); // This does tuple struct
    }; // T = (T1)
    ( impl $($i:ident),* ) => {
        #[allow(non_snake_case)]
        impl<$($i: Type + 'static),*> Type for ($($i),*) {
            const MODULE_PATH: &'static str = module_path!();

            #[allow(unused)]
            fn inline(opts: DefOpts, generics: &[DataType]) -> Result<DataType, ExportError> {
                let mut _generics = generics.iter();

                $(let $i = _generics.next().map(Clone::clone).map_or_else(
                    || {
                        $i::reference(
                            DefOpts {
                                parent_inline: opts.parent_inline,
                                type_map: opts.type_map,
                            },
                            generics,
                        )
                    },
                    Ok,
                )?;)*

                Ok(datatype::TupleType {
                    fields: vec![$($i),*],
                    generics: vec![]
                }.to_anonymous())
            }
        }
    };
    ( $i2:ident $(, $i:ident)* ) => {
        impl_tuple!(impl $i2 $(, $i)* );
        impl_tuple!($($i),*);
    };
    () => {};
}

macro_rules! impl_containers {
    ($($container:ident)+) => {$(
        impl<T: Type> Type for $container<T> {
            const MODULE_PATH: &'static str = module_path!();

            fn inline(opts: DefOpts, generics: &[DataType]) -> Result<DataType, ExportError> {
                generics.get(0).cloned().map_or_else(
                    || {
                        T::inline(
                           opts,
                            generics,
                        )
                    },
                    Ok,
                )
            }

            fn reference(opts: DefOpts, generics: &[DataType]) -> Result<DataType, ExportError> {
                generics.get(0).cloned().map_or_else(
                    || {
                        T::reference(
                            opts,
                            generics,
                        )
                    },
                    Ok,
                )
            }
        }
    )+}
}

macro_rules! impl_as {
    ($($ty:path as $tty:ident)+) => {$(
        impl Type for $ty {
            const MODULE_PATH: &'static str = module_path!();

            fn inline(opts: DefOpts, generics: &[DataType]) -> Result<DataType, ExportError> {
                <$tty as Type>::inline(opts, generics)
            }

            fn reference(opts: DefOpts, generics: &[DataType]) -> Result<DataType, ExportError> {
                <$tty as Type>::reference(opts, generics)
            }
        }
    )+};
}

macro_rules! impl_for_list {
    ($($ty:path as $name:expr)+) => {$(
        impl<T: Type> Type for $ty {
            const MODULE_PATH: &'static str = module_path!();

            fn inline(opts: DefOpts, generics: &[DataType]) -> Result<DataType, ExportError> {
                Ok(DataType::List(Box::new(generics.get(0).cloned().unwrap_or(T::inline(
                    opts,
                    generics,
                )?))))
            }

            fn reference(opts: DefOpts, generics: &[DataType]) -> Result<DataType, ExportError> {
                Ok(DataType::List(Box::new(generics.get(0).cloned().map_or_else(
                    || {
                        T::reference(
                           opts,
                            generics,
                        )
                    },
                    Ok,
                )?)))
            }
        }
    )+};
}

macro_rules! impl_for_map {
    ($ty:path as $name:expr) => {
        impl<K: Type, V: Type> Type for $ty {
            const MODULE_PATH: &'static str = module_path!();

            fn inline(opts: DefOpts, generics: &[DataType]) -> Result<DataType, ExportError> {
                Ok(DataType::Record(Box::new((
                    generics.get(0).cloned().map_or_else(
                        || {
                            K::inline(
                                DefOpts {
                                    parent_inline: opts.parent_inline,
                                    type_map: opts.type_map,
                                },
                                generics,
                            )
                        },
                        Ok,
                    )?,
                    generics.get(1).cloned().map_or_else(
                        || {
                            V::inline(
                                DefOpts {
                                    parent_inline: opts.parent_inline,
                                    type_map: opts.type_map,
                                },
                                generics,
                            )
                        },
                        Ok,
                    )?,
                ))))
            }

            fn reference(opts: DefOpts, generics: &[DataType]) -> Result<DataType, ExportError> {
                Ok(DataType::Record(Box::new((
                    generics.get(0).cloned().map_or_else(
                        || {
                            K::reference(
                                DefOpts {
                                    parent_inline: opts.parent_inline,
                                    type_map: opts.type_map,
                                },
                                generics,
                            )
                        },
                        Ok,
                    )?,
                    generics.get(1).cloned().map_or_else(
                        || {
                            V::reference(
                                DefOpts {
                                    parent_inline: opts.parent_inline,
                                    type_map: opts.type_map,
                                },
                                generics,
                            )
                        },
                        Ok,
                    )?,
                ))))
            }
        }
    };
}
