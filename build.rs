extern crate pkg_config;
extern crate submodules;

use std::process::Command;
use std::fs::File;
use std::env;

fn print_common_rustc_flags(target: &str)
{
  println!("cargo:rustc-link-lib=static=cubeb");
  if target.contains("darwin") {
      println!("cargo:rustc-link-lib=framework=AudioUnit");
  } else if target.contains("androideabi") {
      println!("cargo:rustc-link-lib=OpenSLES");
  } else if target.contains("linux") {
      println!("cargo:rustc-link-lib=asound");
  } else if target.contains("windows") {
      println!("cargo:rustc-link-lib=winmm");
  }
}

fn check_command(cmd: &str) -> bool {
  return Command::new("which").arg(cmd)
                              .status()
                              .unwrap_or_else(|e| {
    panic!("Failed to execute command: {}", e)
  }).success();
}

fn main()
{
  let build_cubeb = env::var("CARGO_FEATURE_BUILD_CUBEB").is_ok();
  let target = env::var("TARGET").unwrap();
  let host = env::var("HOST").unwrap();

  if !build_cubeb {
    if target != host {
      panic!("For cross-builds use the 'build-cubeb' feature.");
    } if !pkg_config::Config::new().find("cubeb").is_ok() {
      panic!("Missing libcubeb. Install it manually or build cult with \
             '--features build-cubeb'.");
    } else {
      print_common_rustc_flags(&target);
    }
    return
  }

  let out_dir = env::var("OUT_DIR").unwrap();

  let cubeb_dir = "cubeb";

  submodules::update().init().run();

  assert!(check_command("autoreconf"), "autoreconf missing!");
  assert!(check_command("automake"), "automake missing!");
  assert!(check_command("pkg-config"), "pkg-config missing!");
  assert!(check_command("libtool"), "libtool missing!");

  assert!(env::set_current_dir(cubeb_dir).is_ok());

  assert!(Command::new("autoreconf").arg("--install")
                                    .status()
                                    .unwrap_or_else(|e| {
    panic!("Failed to execute command: {}", e);
  }).success(), "autoreconf exited with an error.");

  assert!(Command::new("./configure").args(&["--host", &target])
                                     .args(&["--prefix", &out_dir])
                                     .status()
                                     .unwrap_or_else(|e| {
    panic!("Failed to execute command: {}", e);
  }).success(), "./configure exited with an error.");

  assert!(Command::new("make").arg("install")
                              .status()
                              .unwrap_or_else(|e| {
    panic!("Failed to execute command: {}", e);
  }).success(), "make exited with an error.");

  println!("cargo:rustc-link-search=native={}/lib", out_dir);
  print_common_rustc_flags(&target);
}
