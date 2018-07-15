
#[cfg(all(feature = "win32", target_os = "windows"))]
mod build_win32;
#[cfg(all(feature = "win32", target_os = "windows"))]
use build_win32 as inner;

fn main() {
	inner::main()
}

#[cfg(not(any(feature = "win32", feature = "cocoa", feature = "qt5")))]
mod inner {
    pub fn main() {}
}