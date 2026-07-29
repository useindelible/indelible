macro_rules! impl_string_enum {
    ($ty:ty, $label:literal, { $($variant:ident => $name:literal),+ $(,)? }) => {
        impl $ty {
            pub const NAMES: &'static [&'static str] = &[$($name),+];

            pub const fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $name),+
                }
            }
        }

        impl ::std::fmt::Display for $ty {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl ::std::str::FromStr for $ty {
            type Err = String;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                match value {
                    $($name => Ok(Self::$variant),)+
                    other => Err(format!("invalid {}: {other}", $label)),
                }
            }
        }
    };
}

pub(crate) use impl_string_enum;
