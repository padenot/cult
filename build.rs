extern crate pkg_config;
extern crate submodules;
extern crate cmake;

use std::io::{BufRead, BufReader};
use std::fs::File;
use std::env;

fn parse_cubeb_cache(cache: String) -> String {
  let file = File::open(cache.clone()).unwrap_or_else(|e| {
    panic!("Failed to open {}: {}", cache, e);
  });
  let mut reader = BufReader::new(file);
  let mut line = String::new();
  let mut flags = String::new();

  while reader.read_line(&mut line).unwrap() > 0 {
    {
      if line.contains("USE_ALSA:INTERNAL=1") {
        flags.push_str("cargo:rustc-link-lib=asound\n");
      }
      else if line.contains("USE_AUDIOUNIT:INTERNAL=1") {
        flags.push_str("cargo:rustc-link-lib=framework=AudioUnit\n");
      }
      else if line.contains("USE_JACK:INTERNAL=1") {
        flags.push_str("cargo:rustc-link-lib=jack\n");
      }
      else if line.contains("USE_OPENSL:INTERNAL=1") {
        flags.push_str("cargo:rustc-link-lib=OpenSLES\n");
      }
      else if line.contains("USE_PULSE:INTERNAL=1") {
        flags.push_str("cargo:rustc-link-lib=pulse\n");
      }
      else if line.contains("USE_SNDIO:INTERNAL=1") {
        flags.push_str("cargo:rustc-link-lib=sndio\n");
      }
      else if line.contains("USE_WINMM:INTERNAL=1") {
        flags.push_str("cargo:rustc-link-lib=winmm\n");
      }
    }
    line.clear();
  };

  flags
}

fn main()
{
  let build_cubeb = env::var("CARGO_FEATURE_BUILD_CUBEB").is_ok();
  let target = env::var("TARGET").unwrap();
  let host = env::var("HOST").unwrap();

  if !build_cubeb {
    if target != host {
      panic!("For cross-builds use the 'build-cubeb' feature.");
    } else if !pkg_config::Config::new().find("cubeb").is_ok() {
      panic!("Missing libcubeb. Install it manually or build cult with \
             '--features build-cubeb'.");
    }
    /* if using a pre-existing libcubeb, just link against it dynamically */
    println!("cargo:rustc-link-lib=dylib=cubeb");
    return
  }

  submodules::update().init().recursive().run();

  let dst = cmake::build("cubeb");

  println!("cargo:rustc-link-search=native={}", dst.display());
  println!("cargo:rustc-link-lib=static=cubeb");
  println!("cargo:rustc-flags=-l dylib=stdc++");
  print!("{}", parse_cubeb_cache(format!("{}/build/CMakeCache.txt", dst.display())));
}
