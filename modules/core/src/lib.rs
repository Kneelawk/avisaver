#[macro_use]
extern crate tracing;

use const_format::concatcp;
use directories::{BaseDirs, ProjectDirs, UserDirs};
use lazy_static::lazy_static;

pub mod settings;

pub const APPLICATION_QUALIFIER: &str = "com";
pub const APPLICATION_ORGANIZATION: &str = "kneelawk";
pub const APPLICATION_NAME: &str = "avisaver";
pub const APPLICATION_ID: &str = concatcp!(
    APPLICATION_QUALIFIER,
    ".",
    APPLICATION_ORGANIZATION,
    ".",
    APPLICATION_NAME
);
pub const APPLICATION_TITLE: &str = "AviSaver";

lazy_static! {
    pub static ref BASE_DIRS: BaseDirs = BaseDirs::new().expect("Unable to get base directories");
    pub static ref USER_DIRS: UserDirs = UserDirs::new().expect("Unable to get user directories");
    pub static ref PROJECT_DIRS: ProjectDirs = ProjectDirs::from(
        APPLICATION_QUALIFIER,
        APPLICATION_ORGANIZATION,
        APPLICATION_NAME
    )
    .expect("Unable to get project directories");
}
