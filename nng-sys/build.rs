extern crate cmake;

fn main()
{
	#[cfg(feature = "build-nng")]
	{
		let dst = cmake::build("libnng");
		println!("cargo:rustc-link-search=native={}/lib", dst.display());
		println!("cargo:rustc-link-search=native={}/lib64", dst.display());
		println!("cargo:rustc-link-lib=static=nng");
	}
	#[cfg(not(feature = "build-nng"))]
	{
		println!("cargo:rustc-link-lib=dylib=nng");
	}
}
