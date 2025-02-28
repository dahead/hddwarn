use std::{env, fs::{File}, io::{self, Write}, process};
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
    recipient: String,
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
            output.push_str(&format!("DISK {} has {} GB left free space. WARNING!\n", disk.name, disk.free_space));
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
        recipient: "john.doe@example.com".to_string(),
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

// Function to create the autostart registry file
fn create_autostart_helper_files() -> io::Result<()> {
    // Get the path to the current executable
    let executable_path = env::current_exe().unwrap();
    let executable_path_str = executable_path.to_string_lossy().to_string();

    // Create the .reg file content
    let reg_content = format!(
        r#"
[HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run]
"hddwarn"="{}"
"#,
        executable_path_str
    );

    // Path where the .reg file will be saved
    let reg_file_path = Path::new("autostart.reg");

    // Create or open the .reg file and write the content
    let mut reg_file = File::create(reg_file_path)?;
    reg_file.write_all(reg_content.as_bytes())?;
    println!("Auto-start registry file created at autostart.reg");

    Ok(())
}

// Function to create a task scheduler entry
fn create_task_scheduler_entries() -> io::Result<()> {
    // Get the path to the current executable
    let executable_path = env::current_exe().unwrap();
    let executable_path_str = executable_path.to_string_lossy().to_string();

    // XML content for creating a Task Scheduler entry that runs every 24 hours
    let task_xml = format!(
        r#"
<?xml version="1.0" encoding="UTF-8"?>
<Task version="1.3" xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task">
  <RegistrationInfo>
    <Author>hddwarn</Author>
    <Description>Runs hddwarn every 24 hours</Description>
  </RegistrationInfo>
  <Triggers>
    <CalendarTrigger>
      <StartBoundary>2025-02-28T12:00:00</StartBoundary>
      <Repetition>
        <Interval>P1D</Interval>
        <Duration>P1D</Duration>
      </Repetition>
    </CalendarTrigger>
  </Triggers>
  <Actions>
    <Exec>
      <Command>{}</Command>
    </Exec>
  </Actions>
</Task>
"#,
        executable_path_str
    );

    // Path where the XML task file will be saved
    let task_file_path = Path::new("task_scheduler.xml");

    // Create or open the XML file and write the content
    let mut task_file = File::create(task_file_path)?;
    task_file.write_all(task_xml.as_bytes())?;
    println!("Task Scheduler entry XML created at task_scheduler.xml");

    // Register the task using schtasks command
    let schtasks_command = format!(
        "schtasks /create /tn \"hddwarn Task\" /xml \"{}\" /f",
        task_file_path.display()
    );

    // Execute the command to register the task
    let output = process::Command::new("cmd")
        .arg("/C")
        .arg(schtasks_command)
        .output()
        .expect("Failed to execute schtasks command");

    if !output.status.success() {
        eprintln!("Failed to register the task scheduler entry.");
        return Err(io::Error::new(io::ErrorKind::Other, "Failed to register the task"));
    }

    println!("Task scheduler entry successfully created.");
    Ok(())
}

fn main() {

    // Special parameters?
    let args: Vec<String> = env::args().collect();

    if args.len() >= 2 {

        // remember parameter 1
        let param1 = &args[1];

        // Check if the first parameter is "create_autostart_helper_files".
        // That creates a Windows registry file that adds this app to autostart if imported via double click.
        if param1 == "create_autostart_helper_files" {
            if let Err(e) = create_autostart_helper_files() {
                eprintln!("Failed to create auto-start helper files: {}", e);
            }
            return;
        }

        // Check if the first parameter is "create_task_scheduler_entries".
        // This creates an entry in the Task Scheduler so that the app runs every 24h.
        if param1 == "create_task_scheduler_entries" {
            if let Err(e) = create_task_scheduler_entries() {
                eprintln!("Failed to create task-start scheduler entries: {}", e);
            }
            return;
        }
    }

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

    // get hostname for the report
    let hostname_str = hostname::get()
        .map(|hostname| hostname.to_string_lossy().into_owned())
        .unwrap_or_else(|_| String::from("Unknown"));

    // collect report data
    let disks = get_disk_info();
    let mail_content = format_mail_content(&hostname_str, &disks);

    // Print the mail content to the CLI
    println!("\n--- Mail Content ---\n{}", mail_content);

    // Send report via mail
    send_mail(
        &config.mailserver,
        config.port,
        &config.sendmail,
        &config.password,
        &config.recipient,
        "Disk Space Report",
        &mail_content,
    );

}
