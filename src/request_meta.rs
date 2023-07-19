use hudsucker::hyper::Uri;

#[derive(Clone, Debug)]
pub enum RequestType {
    Package,
    Repodata,
    Metalink,
    Mirrorlist,
    Unknown,
}

macro_rules! builder {
    ($name:ident, $buildername:ident, [$($var:ident: $vartype:ty),+ $(,)?]) => {
        #[derive(Clone, Debug)]
        pub struct $name {
            $(
                $var: $vartype,
            )+
        }

        #[derive(Default, Debug)]
        pub struct $buildername {
            $(
                $var: Option<$vartype>,
            )+
        }

        impl $buildername {
            pub fn new() -> Self {
                Default::default()
            }

            $(
            pub fn $var(&mut self, $var: $vartype) {
                self.$var = Some($var);
            }
            )+

            pub fn build(self) -> $name {
                $name {
                    $(
                        $var: self.$var.expect(&format!("{}.build() called with {} not set", stringify!($buildername), stringify!($var))),
                    )+
                }
            }
        }
    };
}

builder!(RequestMetadata, RequestMetadataBuidler, [
    uri: Uri,
    request_type: RequestType,
    view_name: String,
]);