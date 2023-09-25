pub trait GeneratedEntity {
    type Accessor<'a>;
    type AccessorMut<'a>;

    fn write_to(self, v: &mut hecs::EntityBuilder);
}

macro_rules! define_entity {


    (
        tagged $name:ident : ($($extends_name:ident : $extends:ty),*) {
            $(
                $field:ident : $ty:ty
            ),*
        }
    ) => {

        paste::paste! {
            struct [<$name Tag>];
        }

        define_entity!{
            $name : ($($extends_name: $extends)*) {
                tag: paste::paste! {[<$name Tag>]},
                $(
                    $field: $ty
                ),*
            }
        }
    };

    (
            $name:ident : ($($extends_name:ident : $extends:ty),*) {
                $(
                    $field:ident : $ty:ty
                ),*
            }
    ) => {

            paste::paste! {
                #[derive(hecs::Query)]
                struct [<$name Ref>]<'a> {
                    $(
                        $field: &'a $ty,
                    )*
                    $(
                        $extends_name: <$extends as GeneratedEntity>::Accessor<'a>,
                    )*
                }

                #[derive(hecs::Query)]
                struct [<$name RefMut>]<'a> {
                    $(
                        $field: &'a mut $ty,
                    )*
                    $(
                        $extends_name: <$extends as GeneratedEntity>::AccessorMut<'a>,
                    )*
                }
            }


            struct $name {
                $(
                    $field: $ty,
                )*
                $(
                    $extends_name: $extends
                ),*
            }

            impl GeneratedEntity for $name {
                paste::paste! {
                    type Accessor<'a> = [<$name Ref>]<'a>;
                    type AccessorMut<'a> = [<$name RefMut>]<'a>;
                }

                fn write_to(self, v: &mut hecs::EntityBuilder) {
                    $(
                        v.add(self.$field);
                    )*
                    $(
                        self.$extends_name.write_to(v);
                    )*
                }
            }
    };
}
