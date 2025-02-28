use std::{env, process};
use std::fs::File;
use std::io::{self};
use std::io::{BufReader};
use std::path::Path;
use sysinfo::{System, SystemExt, DiskExt};
use lettre::{Message, SmtpTransport, Transport};
use lettre::transport::smtp::authentication::Credentials;
use serde::{Serialize, Deserialize};
extern crate hostname;

#[derive(Serialize, Deserialize)]
struct Config {
    mailserver: String,
    port: u16,
    sendmail: String,
    password: String,
}

struct DiskInfo {
    name: String,
    free_space: u64,
}

fn get_disk_info() -> Vec<DiskInfo> {
    let mut sys = System::new_all();
    sys.refresh_disks_list();
    sys.refresh_disks();

    let mut disks_info = Vec::new();
    for disk in sys.disks() {
        let free_space = disk.available_space() / 1_073_741_824; // Convert bytes to GB
        disks_info.push(DiskInfo {
            name: disk.mount_point().to_string_lossy().into_owned(),
            free_space,
        });
    }
    disks_info
}

fn format_mail_content(server_name: &str, disks: &[DiskInfo]) -> String {
    let mut output = format!("Servername: {}\n", server_name);
    for disk in disks {
        if disk.free_space <= 10 {
            output.push_str(&format!("WARNING! DISK {} has {} GB left free space.\n", disk.name, disk.free_space));
        } else {
            output.push_str(&format!("DISK {} has {} GB left free space.\n", disk.name, disk.free_space));
        }
    }
    output
}

fn create_default_config() -> io::Result<()> {
    let default_config = Config {
        mailserver: "smtp.example.com".to_string(),
        port: 587,
        sendmail: "youremail@example.com".to_string(),
        password: "yourpassword".to_string(),
    };

    let config_path = Path::new("config.json");
    let config_file = File::create(config_path)?;

    serde_json::to_writer(config_file, &default_config)?;
    println!("Default config created at config.json");
    Ok(())
}

fn read_config() -> Option<Config> {
    let config_path = "config.json";
    let file = File::open(config_path).ok()?;
    let reader = BufReader::new(file);

    let config: Config = serde_json::from_reader(reader).ok()?;
    Some(config)
}

fn send_mail(mailserver: &str, port: u16, sendmail: &str, password: &str, recipient: &str, subject: &str, body: &str) {
    let email = Message::builder()
        .from(sendmail.parse().unwrap())
        .to(recipient.parse().unwrap())
        .subject(subject)
        .body(body.to_string())
        .unwrap();

    let creds = Credentials::new(sendmail.to_string(), password.to_string());
    let mailer = SmtpTransport::relay(mailserver)
        .unwrap()
        .port(port)
        .credentials(creds)
        .build();

    match mailer.send(&email) {
        Ok(_) => println!("Email sent successfully to {}", recipient),
        Err(e) => eprintln!("Failed to send email: {}", e),
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <recipient_email>", args[0]);
        process::exit(1);
    }
    let recipient = &args[1];

    // Check for config.json
    let config = if let Some(config) = read_config() {
        config
    } else {
        eprintln!("Config file not found, creating default config...");
        if let Err(e) = create_default_config() {
            eprintln!("Failed to create default config: {}", e);
        }
        std::process::exit(1);
    };

    let hostname_str = hostname::get()
        .map(|hostname| hostname.to_string_lossy().into_owned())
        .unwrap_or_else(|_| String::from("Unknown"));

    let disks = get_disk_info();
    let mail_content = format_mail_content(&hostname_str, &disks);

    // Print the mail content to the CLI
    println!("\n--- Mail Content ---\n{}", mail_content);

    send_mail(
        &config.mailserver,
        config.port,
        &config.sendmail,
        &config.password,
        recipient,
        "Disk Space Report",
        &mail_content,
    );

}
