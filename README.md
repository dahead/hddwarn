### About

This application collects the remaining space of each hard disk on the current system, creates a little report and sends that to a recipient via SMTP.

### Config

After the first start of the application it creates a default JSON config file in the app directory.
This config.json is used for storing the SMTP Server details like servername, port, send mail adress and password.

### Usage

./hddwarn it@company.com

### Output

The application prints the report to the CLI.

The output looks something like this:

```
DISK / has 258 GB left free space.
WARNING! DISK /boot has 0 GB left free space.
DISK /mnt/2TB has 70 GB left free space.
```

### Todo

- Don't store the password in the JSON config ;-)