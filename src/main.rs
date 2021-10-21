use std::{env, fs, thread, process};
use std::fs::File;
use clap::{Arg, App};
use std::io::{Write, Read};
use std::net::TcpListener;
use serde_json::Value;
use std::path::PathBuf;
use tar::{Builder, Archive};
use std::str;
use curl::easy::Easy;

fn main() {
    let matches = App::new("verzcon")
        .version("1.0")
        .about("Version control")
        .arg(Arg::with_name("host")
            .short("h")
            .long("host")
            .help("host the code"))
        .arg(Arg::with_name("new")
            .short("n")
            .long("new")
            .help("make a new example config"))
        .arg(Arg::with_name("update")
            .short("u")
            .long("update")
            .help("update code to the newest"))
        .get_matches();
    let mut dir = env::current_exe().unwrap();
    dir.pop();
    dir.push("config.json");
    if dir.exists() && !matches.is_present("new") {
        let json: Value = serde_json::from_str(fs::read_to_string(dir).unwrap().as_str()).unwrap();
        if matches.is_present("host") {
            ip();
            let listener = TcpListener::bind("0.0.0.0:1754").unwrap();
            for stream in listener.incoming() {
                let mut stream = stream.unwrap();
                let mut buffer = [0; 1024];
                stream.read(&mut buffer).unwrap();
                let mut get_next = false;
                let mut data = "".to_string();
                let mut length = 0;
                for i in String::from_utf8_lossy(&buffer[..]).split("\n") {
                    if get_next == true && !i.trim().is_empty() {
                        data.push_str(i.trim());
                    }
                    if i.starts_with("Content-Length: ") {
                        length = i.replace("Content-Length: ", "").trim().parse().unwrap();
                    }
                    if i.trim() == "" {
                        get_next = true;
                    }
                }
                data = (&data[..length]).to_string();
                if data.to_string() == "version" {
                    let mut ar = Builder::new(Vec::new());
                    ar.append_dir_all(PathBuf::from(json["Folder"].to_string().trim_matches('\"').to_string()).file_name().unwrap(), json["Folder"].to_string().trim_matches('\"').to_string()).unwrap();
                    let contents = format!("{:?}", md5::compute(String::from_utf8_lossy(&*ar.into_inner().unwrap()).to_string()));
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
                        contents.len(),
                        contents
                    );
                    stream.write(response.as_bytes()).unwrap();
                }
                else if data.to_string() == "cont" {
                    let mut ar = Builder::new(Vec::new());
                    ar.append_dir_all(PathBuf::from(json["Folder"].to_string().trim_matches('\"').to_string()).file_name().unwrap(), json["Folder"].to_string().trim_matches('\"').to_string()).unwrap();
                    let contents = &*ar.into_inner().unwrap();
                    let contents = String::from_utf8_lossy(contents).to_string();
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
                        contents.len(),
                        contents
                    );
                    stream.write(response.as_bytes()).unwrap();
                }
                else {
                    stream.write("HTTP/1.1 404\nContent-Length: 0".as_bytes()).unwrap();
                }
                stream.flush().unwrap();
            }
        }
        else {
            let mut dst = Vec::new();
            let mut easy = Easy::new();
            let mut data = "version".as_bytes();
            easy.url(&[json["Ip"].to_string().trim_matches('\"').to_string(), ":1754".to_string()].join("")).unwrap();
            easy.post(true).unwrap();
            easy.post_field_size(data.len() as u64).unwrap();
            let mut transfer = easy.transfer();
            transfer
                .read_function(|buf| {
                    let len = std::cmp::min(buf.len(), data.len());
                    buf[..len].copy_from_slice(&data[..len]);
                    data = &data[len..];
                    Ok(len)
                })
                .unwrap();
            transfer
                .write_function(|buf| {
                    dst.extend_from_slice(buf);
                    Ok(buf.len())
                })
                .unwrap();
            while transfer.perform().is_err() {}
            drop(transfer);
            if dst.iter().map(|&c| c as char).collect::<String>().trim() == "" {
                println!("A error happened try again");
                process::exit( 520);
            }
            let returns = dst.iter().map(|&c| c as char).collect::<String>();
            let mut ar = Builder::new(Vec::new());
            ar.append_dir_all(PathBuf::from(json["Folder"].to_string().trim_matches('\"').to_string()).file_name().unwrap(), json["Folder"].to_string().trim_matches('\"').to_string()).unwrap();
            if returns == format!("{:?}", md5::compute(String::from_utf8_lossy(&*ar.into_inner().unwrap()).to_string())) {
                println!("up to date");
            }
            else {
                println!("not up to date");
                if matches.is_present("update") {
                    let mut dst = Vec::new();
                    let mut easy = Easy::new();
                    let mut data = "cont".as_bytes();
                    easy.url(&[json["Ip"].to_string().trim_matches('\"').to_string(), ":1754".to_string()].join("")).unwrap();
                    easy.custom_request("POST");
                    easy.post_field_size(data.len() as u64).unwrap();
                    let mut transfer = easy.transfer();
                    transfer
                        .read_function(|buf| {
                            let len = std::cmp::min(buf.len(), data.len());
                            buf[..len].copy_from_slice(&data[..len]);
                            data = &data[len..];
                            Ok(len)
                        })
                        .unwrap();
                    transfer
                        .write_function(|buf| {
                            dst.extend_from_slice(buf);
                            Ok(buf.len())
                        })
                        .unwrap();
                    while transfer.perform().is_err() {}
                    drop(transfer);
                    if dst.iter().map(|&c| c as char).collect::<String>().trim() == "" {
                        println!("A error happened try again");
                        process::exit( 520);
                    }
                    let returns = dst.iter().map(|&c| c as char).collect::<String>();
                    let mut ar = Archive::new(returns.as_bytes());
                    ar.unpack(PathBuf::from(json["Folder"].to_string().trim_matches('\"').to_string()).pop().to_string()).unwrap();
                }
            }
        }
    }
    else {
        if matches.is_present("host") {
            create_file(dir.display().to_string(), "{\n\t\"Folder\":\"path ex. c:\\bob or /home/bob/billy\"\n}".to_string());
        }
        else {
            create_file(dir.display().to_string(), "{\n\t\"Ip\": \"the ip ex 127.0.0.1 or 139.130.4.5\",\n\t\"Folder\": \"path ex. c:\\bob or /home/bob/billy\"\n}".to_string());
        }
        println!("In {} is your config file that you need to edit, please do so", dir.display().to_string());
    }
}

fn create_file(file:String, cont:String) -> std::io::Result<()> {
    let mut file = File::create(file)?;
    file.write_all(cont.as_bytes())?;
    Ok(())
}

#[tokio::main]
async fn ip() {
    // Attempt to get an IP address and print it.
    if let Some(ip) = public_ip::addr().await {
        println!("public ip address: {:?}\n", ip);
    } else {
        println!("couldn't get an IP address\n");
    }
}