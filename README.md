### About

This application collects the remaining disk space of each hard disk and partition on the current system.
Then it creates a small report and sends that to a recipient via SMTP.
It marks free space less than 10% as a "Warning".

### Config

The application uses a config.json which stores information for sending the report by mail.
If there is no config, the application creates one when run for the first time.

At the moment the SMTP mailbox password is stored in plaintext in the config file which is not optimal.

### Usage

./hddwarn

### Output

The application prints the report to the CLI.

The output looks something like this:

```
DISK / has 258 GB left free space.
DISK /boot has 0 GB left free space. WARNING!
DISK /mnt/2TB has 70 GB left free space.
```

### Todo

- Don't store the password in the JSON config ;-)