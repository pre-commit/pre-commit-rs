use std::env;
use std::ffi;
use std::ops::Not;

pub(crate) fn var_os_nonempty<K>(k: K) -> Option<ffi::OsString>
where
    K: AsRef<ffi::OsStr>,
{
    env::var_os(k).and_then(|s| s.is_empty().not().then_some(s))
}
