pub mod handler;

pub trait EventWrapper<T>: Sized {
    fn wrap(self) -> T;
    fn unwrap(value: &T) -> Option<&Self>;
}

pub trait EventCollection: Send + Sync {
    fn inner_id(&self) -> std::any::TypeId;
}

macro_rules! events {
    (
        $event_collection_name:ident {
            $(
                $event:ty
            ),*
        }
    ) => {
        paste::paste! {

            pub enum $event_collection_name {
                $(
                    [<$event>]($event)
                ),*
            }

            impl EventCollection for $event_collection_name {
                fn inner_id(&self) -> core::any::TypeId {
                    use std::any::Any;
                    match self {
                        $(
                            Self::$event(value) => {
                                value.type_id()
                            }
                        )*
                    }
                }
            }

            $(
                impl EventWrapper<$event_collection_name> for $event {

                    fn wrap(self) -> $event_collection_name {
                        $event_collection_name::$event(self)
                    }

                    fn unwrap(v: &$event_collection_name) -> Option<&Self> {
                        if let $event_collection_name::$event(v) = v {
                            Some(v)
                        } else {
                            None
                        }
                    }
                }
            )*
        }

    };
}