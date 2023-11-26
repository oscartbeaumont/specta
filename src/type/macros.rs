macro_rules! impl_primitives {
    ($($i:ident)+) => {$(
        impl Type for $i {
            fn inline(_: &mut TypeMap) -> DataType {
                DataType::Primitive(datatype::PrimitiveType::$i)
            }
        }
    )+};
}

macro_rules! impl_tuple {
    ( impl $($i:ident),* ) => {
        #[allow(non_snake_case)]
        impl<$($i: Type + 'static),*> Type for ($($i,)*) {
            #[allow(unused)]
            fn inline(type_map: &mut TypeMap) -> DataType {
                // let mut _generics = generics.iter();

                $(let $i = $i::reference(type_map).inner;)*

                datatype::TupleType {
                    elements: vec![$($i),*],
                }.to_anonymous()
            }
        }
    };
    ( $i2:ident $(, $i:ident)* ) => {
        impl_tuple!(impl $($i),* );
        impl_tuple!($($i),*);
    };
    () => {};
}

macro_rules! impl_containers {
    ($($container:ident)+) => {$(
        impl<T: Type> Type for $container<T> {
            fn inline(type_map: &mut TypeMap) -> DataType {
                T::inline(type_map)
            }

            fn reference(type_map: &mut TypeMap) -> Reference {
                Reference {
                    inner: T::reference(type_map).inner,
                    // generics.get(0).cloned().unwrap_or_else(
                    //     || T::reference(type_map).inner,
                    // ),
                }
            }
        }

        impl<T: NamedType> NamedType for $container<T> {
	        const SID: SpectaID = T::SID;
	        const IMPL_LOCATION: ImplLocation = T::IMPL_LOCATION;

            fn named_data_type(type_map: &mut TypeMap) -> NamedDataType {
                T::named_data_type(type_map)
            }

            fn definition_named_data_type(type_map: &mut TypeMap) -> NamedDataType {
                T::definition_named_data_type(type_map)
            }
        }

        impl<T: Flatten> Flatten for $container<T> {}
    )+}
}

macro_rules! impl_as {
    ($($ty:path as $tty:ident)+) => {$(
        impl Type for $ty {
            fn inline(type_map: &mut TypeMap) -> DataType {
                <$tty as Type>::inline(type_map)
            }

            fn reference(type_map: &mut TypeMap) -> Reference {
                <$tty as Type>::reference(type_map)
            }
        }
    )+};
}

macro_rules! impl_for_list {
    ($($ty:path as $name:expr)+) => {$(
        impl<T: Type> Type for $ty {
            fn inline(type_map: &mut TypeMap) -> DataType {
                DataType::List(List {
                    ty: Box::new(T::inline(
                        type_map,
                    )),
                    length: None,
                })
            }

            fn reference(type_map: &mut TypeMap) -> Reference {
                Reference {
                    inner: DataType::List(List {
                        ty: Box::new(T::reference(type_map).inner),
                        length: None,
                    }),
                }
            }
        }
    )+};
}

macro_rules! impl_for_map {
    ($ty:path as $name:expr) => {
        impl<K: Type, V: Type> Type for $ty {
            fn inline(type_map: &mut TypeMap) -> DataType {
                DataType::Map(Box::new((K::inline(type_map), V::inline(type_map))))
            }

            fn reference(type_map: &mut TypeMap) -> Reference {
                Reference {
                    inner: DataType::Map(Box::new((
                        K::reference(type_map).inner,
                        V::reference(type_map).inner,
                    ))),
                }
            }
        }
    };
}
