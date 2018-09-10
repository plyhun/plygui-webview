#[cfg(all(feature = "win32", target_os = "windows"))]
mod build_win32;
#[cfg(all(feature = "win32", target_os = "windows"))]
use build_win32 as inner;

#[cfg(all(feature = "cocoa_", target_os = "macos"))]
mod build_cocoa;
#[cfg(all(feature = "cocoa_", target_os = "macos"))]
use build_cocoa as inner;

fn main() {
	inner::main()
}

#[cfg(not(any(feature = "win32", feature = "cocoa_", feature = "qt5")))]
mod inner {
    pub fn main() {}
}