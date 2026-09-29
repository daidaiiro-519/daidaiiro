use app_core::name::f;
use app_stray::x;
use app_gen::made;
#[cfg(feature = "fast")]
use app_core::fast::f as g;
#[cfg(not(feature = "fast"))]
use app_core::slow::f as g;
include!(concat!(env!("OUT_DIR"), "/gen.rs"));
