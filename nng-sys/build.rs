fn main()
{
	#[cfg(feature = "build-nng")]
	{
		let dst = cmake::Config::new("libnng")
			.define("NNG_TESTS", "OFF")
			.define("NNG_TOOLS", "OFF")
			.define("NNG_ENABLE_NNGCAT", "OFF")
			.define("NNG_ENABLE_COVERAGE", "OFF")
			.build();
		println!("cargo:rustc-link-search=native={}/lib", dst.display());
		println!("cargo:rustc-link-search=native={}/lib64", dst.display());
		println!("cargo:rustc-link-lib=static=nng");
	}
	#[cfg(not(feature = "build-nng"))]
	{
		println!("cargo:rustc-link-lib=dylib=nng");
	}
}
