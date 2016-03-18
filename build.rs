extern crate pkg_config;
extern crate submodules;

use std::process::Command;
use std::io::{BufRead, BufReader};
use std::fs::File;
use std::env;

fn check_command(cmd: &str) -> bool {
  return Command::new("which").arg(cmd)
                              .status()
                              .unwrap_or_else(|e| {
    panic!("Failed to execute command: {}", e)
  }).success();
}

fn parse_cubeb_config_h(config_h: &str) -> String {
  let file = File::open(config_h).unwrap_or_else(|e| {
    panic!("Failed to open config.h: {}", e);
  });
  let mut reader = BufReader::new(file);
  let mut line = String::new();
  let mut flags = String::new();

  while reader.read_line(&mut line).unwrap() > 0 {
    {
      let l = line.trim();
      if l.starts_with("#define") {
        match &l[8..] {
          "HAVE_ALSA_ASOUNDLIB_H 1" => {
            flags.push_str("cargo:rustc-link-lib=asound\n");
          },
          "HAVE_AUDIOUNIT_AUDIOUNIT_H 1" => {
            flags.push_str("cargo:rustc-link-lib=framework=AudioUnit\n");
          },
          "HAVE_DSOUND_H 1" => {
            flags.push_str("cargo:rustc-link-lib=dsound\n");
          },
          "HAVE_PULSE_PULSEAUDIO_H 1" => {
            flags.push_str("cargo:rustc-link-lib=pulse\n");
          },
          "HAVE_SLES_OPENSLES_H 1" => {
            flags.push_str("cargo:rustc-link-lib=OpenSLES\n");
          },
          "HAVE_SNDIO_H 1" => {
            flags.push_str("cargo:rustc-link-lib=sndio\n");
          },
          "HAVE_WINDOWS_H 1" => {
            flags.push_str("cargo:rustc-link-lib=winmm\n");
          },
          _ => {}
        }
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
  println!("cargo:rustc-link-lib=static=cubeb");
  print!("{}", parse_cubeb_config_h("config.h"));
}
