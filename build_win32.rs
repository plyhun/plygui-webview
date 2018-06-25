extern crate cc;

pub fn main() {
	let mut cc_build = cc::Build::new();
	cc_build.opt_level(3)
        .cpp(true)
        //.debug(true)
        //.flag("-fkeep-inline-functions")
        .warnings(false)
        .flag("-std=c++14")
		.include("sys/win32")
		.file("sys/win32/webview.cpp")
		.compile("webview");
}