use std::fmt::Debug;

pub trait ResultExt {
    fn warn(self, msg: &str);

    fn error(self, msg: &str);
}

impl<T, E: Debug> ResultExt for Result<T, E> {
    fn warn(self, msg: &str) {
        if let Err(err) = self {
            warn!("{msg}: {err:?}");
        }
    }

    fn error(self, msg: &str) {
        if let Err(err) = self {
            error!("{msg}: {err:?}");
        }
    }
}
