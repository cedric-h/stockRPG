#![cfg(feature = "nightly")]
use serde::Serialize;

const LEFT_STRING: &'static str = "{\"";
const RIGHT_STRING: &'static str = "}";
const MIDDLE_STRING: &'static str = "\": ";

pub trait MaybeSerialize {
    fn maybe_serialize(&self) -> Option<String>;
}

impl<T> MaybeSerialize for T
where
    T: ?Sized,
{
    default fn maybe_serialize(&self) -> Option<String> {
        None
    }
}

impl<T> MaybeSerialize for T
where
    T: Serialize,
{
    fn maybe_serialize(&self) -> Option<String> {
        let type_name = unsafe { ::std::intrinsics::type_name::<T>() };
        let mut last_colon_index = 0;
        for (index, letter) in type_name.char_indices() {
            if letter == ':' {
                last_colon_index = index;
            }
        }
        let (_, type_name) = type_name.split_at(last_colon_index + 1);
        Some(
            LEFT_STRING.to_owned() +
                type_name +
                MIDDLE_STRING +
                &serde_json::to_string(&self).unwrap() +
                RIGHT_STRING,
        )
    }
}
