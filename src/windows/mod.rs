#[macro_use]
#[macro_export]
macro_rules! get_object {
    ($name:ident, $T:ty, $b:ident) => {
        let $name: $T = $b.get_object(stringify!($name)).unwrap();
    };
}

pub mod mainwindow;
pub use self::mainwindow::MainWindow;

pub mod albumwindow;
pub use self::albumwindow::AlbumWindow;
