/*
 * Created on Tue Dec 11 2018
 * Author Adrian Challinor FRAS
 *
 * Copyright (c) 2018 Osiris Consultants Ltd ALL RIGHTS RESERVED
 */

#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] 
extern crate rocket;

extern crate clap;
extern crate chrono;
extern crate hostname;
extern crate libc;
extern crate uname;
extern crate uptime_lib;
extern crate systemstat;
extern crate time;

use clap::{App, Arg};

use libc::c_char;
use libc::statvfs;
use rocket::config::{Config, Environment, Limits, LoggingLevel};
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::net::{TcpStream, UdpSocket};

use std::mem;
use std::path::Path;
use std::process;
use std::process::Command;
use std::thread;
use std::time::Duration;
use systemstat::{Platform, System};
use uname::uname;

fn main() {
    let cmd_arguments = App::new("monitor")
        .version("0.2")
        .author("Adrian Challinor <adrian.challinor at osiris.co.uk>")
        .about("Remote monitor server, mainly for CONKY")
        .arg(
            Arg::with_name("address")
                .short('a')
                .long("address")
                .help("The IP address to bind to")
                .default_value("0.0.0.0"),
        )
        .arg(
            Arg::with_name("port")
                .short('p')
                .long("port")
                .help("The IP port to bind to")
                .default_value("9000"),
        )
        .arg(
            Arg::with_name("sensors")
                .short('s')
                .long("sensors")
                .help("List available sensors and exit"),
        )
        .arg(
            Arg::with_name("workers")
                .short('w')
                .long("workers")
                .help("The number of concurrent worker threads")
                .default_value("5"),
        )
        .get_matches();

    let address = cmd_arguments.value_of("address").unwrap();
    let port: u16 = cmd_arguments
        .value_of("port")
        .unwrap()
        .parse::<u16>()
        .unwrap();
    let workers: u16 = cmd_arguments
        .value_of("workers")
        .unwrap()
        .parse::<u16>()
        .unwrap();

    if cmd_arguments.is_present("sensors") {
        show_sensors(port);
        process::exit(1);
    }

    let config = Config::build(Environment::Production)
        .log_level(LoggingLevel::Critical)
        .address(address)
        .port(port)
        .workers(workers)
        .limits(Limits::new().limit("forms", 8))
        .unwrap();

    rocket::custom(config)
        .mount("/status", routes![status])
        .mount("/name", routes![name])
        .mount("/os/name", routes![os_name])
        .mount("/os/version", routes![os_version])
        .mount("/os/versionname", routes![os_vernamename])
        .mount("/os/codename", routes![os_codename])
        .mount("/temp", routes![temp])
        .mount("/sensors", routes![sensors])
        .mount("/uptime", routes![uptime])
        .mount("/aptcheck", routes![aptcheck])
        .mount("/aptcheckbrief", routes![aptcheckbrief])
        .mount("/reboot", routes![reboot])
        .mount("/disk", routes![disk])
        .mount("/loadavg", routes![load])
        .mount("/cpuload", routes![cpuload])
        .mount("/cpu", routes![cpu])
        .mount("/boot", routes![boot])
        .mount("/uname", routes![unameinfo])
        .mount("/memory", routes![memory])
        .mount("/ip", routes![local_ip])
        .mount("/port", routes![port])
        .register(catchers![not_found])
        .launch();
}

#[get("/")]
fn local_ip() -> String {
    let socket = match UdpSocket::bind("0.0.0.0:0") {
        Ok(s) => s,
        Err(_) => return "None".to_string(),
    };

    match socket.connect("8.8.8.8:80") {
        Ok(()) => (),
        Err(_) => return "None".to_string(),
    };

    match socket.local_addr() {
        Ok(addr) => return addr.ip().to_string(),
        Err(_) => return "None".to_string(),
    };
}

fn check_port_by_ip(ipaddr: &str, port: u16) -> bool {
    match TcpStream::connect((ipaddr, port)) {
        Ok(_) => true,
        Err(_) => false
    }
}

#[get("/<port>")]
fn port(port: u16) -> String  {
    let mut found = check_port_by_ip("0.0.0.0", port);
    if !found {found = check_port_by_ip("localhost", port) };
    if !found {found = check_port_by_ip(&local_ip(), port) };

    match found {
        true => format!("Up"),
        false => format!("Down")
    }
}

#[get("/<param>")]
fn unameinfo(param: String) -> String {
    let info = uname().unwrap();
    format!(
        "{}",
        match param.as_str() {
            "n" => info.nodename,
            "s" => info.sysname,
            "r" => info.release,
            "v" => info.version,
            "m" => info.machine,
            _ => "Param?".to_string(),
        }
    )
}

#[get("/")]
fn memory() -> String {
    let sys = System::new();
    match sys.memory() {
        Ok(mem) => format!("\nMemory total: {} free: {} )", mem.total, mem.free),

        Err(x) => format!("\nMemory error: {}", x),
    }
}

#[get("/")]
fn boot() -> String {
    let sys = System::new();
    match sys.boot_time() {
        Ok(boot_time) => format!("\nBoot time: {}", boot_time),
        Err(x) => format!("\nBoot time: error: {}", x),
    }
}

#[get("/")]
fn load() -> String {
    let sys = System::new();
    match sys.load_average() {
        Ok(loadavg) => format!(
            "\nLoad average: {} {} {}",
            loadavg.one, loadavg.five, loadavg.fifteen
        ),
        Err(x) => format!("\nLoad average: error: {}", x),
    }
}

#[get("/")]
fn cpuload() -> String {
    let sys = System::new();
    match sys.cpu_load_aggregate() {
        Ok(cpu) => {
            thread::sleep(Duration::from_secs(1));
            let cpu = cpu.done().unwrap();
            format!(
                "CPU load: {}% user, {}% nice, {}% system, {}% intr, {}% idle ",
                (cpu.user * 100.0) as i64,
                (cpu.nice * 100.0) as i64,
                (cpu.system * 100.0) as i64,
                (cpu.interrupt * 100.0) as i64,
                (cpu.idle * 100.0) as i64
            )
        }
        Err(x) => format!("\nCPU load: error: {}", x),
    }
}

#[get("/")]
fn cpu() -> String {
    let sys = System::new();
    match sys.cpu_load_aggregate() {
        Ok(cpu) => {
            thread::sleep(Duration::from_secs(1));
            let cpu = cpu.done().unwrap();
            format!("{}", 100 - (cpu.idle * 100.0) as i64)
        }
        Err(x) => format!("\nCPU error: {}", x),
    }
}

#[get("/<mountpoint>/<param>")]
fn disk(mountpoint: String, param: String) -> String {
    let mut info: statvfs = unsafe { mem::zeroed() };
    let target = if mountpoint == "root" {
        format!("/\0")
    } else {
        format!("/{}\0", mountpoint)
    };
    let result = unsafe { statvfs(target.as_ptr() as *const c_char, &mut info) };

    match result {
        0 => match param.as_str() {
            "avail" => format!("{}", byte_size(info.f_bavail as u64 * info.f_bsize as u64)),
            "total" => format!("{}", byte_size(info.f_blocks as u64 * info.f_bsize as u64)),
            "free" => format!("{}", byte_size(info.f_bfree as u64 * info.f_bsize as u64)),
            "freep" => format!("{}%", info.f_bfree as u64 * 100 / info.f_blocks as u64),
            "usedp" => format!(
                "{}%",
                100 - (info.f_bfree as u64 * 100 / info.f_blocks as u64)
            ),
            "files" => format!("{}", info.f_files as u64),
            _ => format!("unknown param"),
        },
        _ => format!("io error on disk: {}", target),
    }
}

fn byte_size(ival: u64) -> String {
    let mut val = ival as f64;
    let units = ["B", "KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"];
    let mut p = 0;
    while val > 1024.0 {
        p = p + 1;
        val = val / 1024.0;
    }

    format!("{} {}", val.round() as u64, units[p])
}

#[get("/")]
fn aptcheck() -> String {
    let output = {
        Command::new("/usr/lib/update-notifier/apt-check")
            .arg("--human-readable")
            .output()
            .expect("Failed")
    };
    let s = String::from_utf8_lossy(&output.stdout);

    s.to_string()
}

#[get("/")]
fn aptcheckbrief() -> String {
    let output = {
        Command::new("/usr/lib/update-notifier/apt-check")
            .output()
            .expect("Failed")
    };
    let apt = String::from_utf8_lossy(&output.stderr);
    let mut split = apt.split(";");
    format!("Installable: {} - Security: {}", split.next().unwrap(), split.next().unwrap())
}

#[get("/")]
fn reboot() -> String {
    let mut s = "";
    if Path::new("/var/run/reboot-required").exists() {
        s = "REBOOT REQUIRED";
    }
    s.to_string()
}

#[get("/")]
fn uptime() -> String {
    match uptime_lib::get() {
        Ok(uptime) => {
            
            let s: u64 = (uptime.as_secs_f64() as f64 ) as u64;
            let m: u64 = s / 60;
            let sd = s - (m * 60);
            let h: u64 = m / 60;
            let md = m - (h * 60);
            let d = h / 24;
            let hd = h - (d * 24);

            format!("{}d {}h {}m {}s", d, hd, md, sd)
        }
        Err(err) => format!("uptime: {}", err),
    }
}

#[get("/<chipname>/<param>")]
fn sensors(chipname: String, param: String) -> String {
    let mut rc = "404-Feature not found".to_string();
    if Path::new("/usr/bin/sensors").exists() {
        let output = {
            Command::new("/usr/bin/sensors")
                .arg("-A")
                .output()
                .expect("Failed")
        };
        let mut device: &str = "";
        String::from_utf8(output.stdout)
            .unwrap()
            .lines()
            .for_each(|x| {
                if !x.is_empty() && !x.starts_with(' ') && !x.starts_with('(') {
                    if x.contains(":") {
                        let v: Vec<&str> = x.split(':').collect();
                        let pname = v[0]
                            .replace("/", "_")
                            .replace(" ", "_")
                            .replace("+", "")
                            .replace(".", "_");

                        if chipname.eq_ignore_ascii_case(device)
                            && param.eq_ignore_ascii_case(pname.as_str())
                        {
                            let vals: Vec<&str> = v[1].trim().split(' ').collect();
                            rc = format!("{}", vals[0]);
                        }
                    } else {
                        device = x;
                    }
                }
            });
    }
    rc
}

#[get("/<id>")]
fn temp(id: String) -> String {
    let sensor = format!("/sys/devices/virtual/thermal/thermal_zone{}/temp", id);
    let f = match File::open(sensor) {
        Ok(file) => file,
        Err(_) => return format!("404 - Not odroid XU4 "),
    };
    let mut file = BufReader::new(&f);
    let mut input = String::new();
    file.read_line(&mut input).expect("Sensor not accessible");
    let line = input.trim();
    println!("Sensor {} = <{}>", id, line);

    let temp = line.parse::<i32>().unwrap() / 1000;
    format!("{}", temp)
}

#[get("/")]
fn os_name() -> String {
    get_param("NAME")
}

#[get("/")]
fn os_version() -> String {
    get_param("VERSION_ID")
}

#[get("/")]
fn os_codename() -> String {
    get_param("UBUNTU_CODENAME")
}

#[get("/")]
fn os_vernamename() -> String {
    get_param("VERSION")
}

fn get_param(key: &str) -> String {
    let f = match File::open("/etc/os-release") {
        Ok(file) => file,
        Err(e) => return format!("404 - os-release {} ", e),
    };

    let file = BufReader::new(&f);
    for line in file.lines() {
        let l = line.unwrap();
        let part: Vec<&str> = l.split('=').collect();
        if part[0] == key {
            return part[1].to_string().replace(r#"""#, "");
        }
    }
    format!("404 - key {} not found", key)
}

/// Basic status
#[get("/")]
fn status() -> String {
    format!("Status - Running - {}", env!("CARGO_PKG_VERSION"))
}

/// Get the nost name
#[get("/")]
fn name() -> String {
    hostname::get().unwrap().to_str().unwrap().to_string()
}

/// Catch 404 errors
#[catch(404)]
fn not_found() -> String {
    format!("404 - Sorry, I can't do that Dave.")
}

fn show_sensors(port: u16) {
    let mut found = false;
    let hostname = name();
    println!("{} : Listing all sensors in conky monitor format", hostname);
    println!("");

    if Path::new("/usr/bin/sensors").exists() {
        let output = {
            Command::new("/usr/bin/sensors")
                .arg("-A")
                .output()
                .expect("Failed")
        };
        let mut device: &str = "";
        String::from_utf8(output.stdout)
            .unwrap()
            .lines()
            .for_each(|x| {
                if !x.is_empty() && !x.starts_with(' ') && !x.starts_with('(') {
                    if x.contains(":") {
                        found = true; 
                        let v: Vec<&str> = x.split(':').collect();
                        let param = v[0]
                            .replace("/", "_")
                            .replace(" ", "_")
                            .replace("+", "")
                            .replace(".", "_");
                        let vals: Vec<&str> = v[1].trim().split(' ').collect();
                        let val = vals[0];
                        println!(
                            "http://{}:{}/sensors/{}/{} = {}",
                            hostname, port, device, param, val
                        );
                    } else {
                        device = x;
                    }
                }
            });
    }
    if !found {
        println!("No sensors found");
    }
}
