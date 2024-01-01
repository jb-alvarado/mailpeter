# mailpeter

Simple mailer and API for contact forms.

**Warning: This project is in a very early stage, do not use it in production!**

## Configuration

Config format is Toml, content of mailpeter.toml should look like:

```Toml
log_keep_count = 10                         # How many log files should be kept until they are removed from the system.
log_level = "debug"                         # What level to log.
log_size_mb = 1                             # The size of the log file until a new log file is created.
log_to_file = false                         # Log to file, or to console.
reverse_proxy_ip = "127.0.0.1"              # IP from reverse proxy, I exists
limit_request_seconds = 30                  # Limit the requests to protect from spamming. 0 for disable rate limit.
max_attachment_size_mb = 5.0                # Maximum size fro attachments.
routes = ["text_only", "with_attachments"]  # Which routes should be provided.
mail_archive = "/var/mail/mailpeter"        # Backup mails in folder, leave it empty for no backup.

[mail]
smtp = "smtp.example.org"
port = 587
user = "info@example.org"
password = "super-secure-mail-password"
starttls = true
alias = ""                                  # Send to an alias, useful for system mail if the recipient is root, for example.

[[mail.recipients]]
direction = "contact"
mails = ["info@example.org", "office@example.org"]

[[mail.recipients]]
direction = "order"
mails = ["shop@example.org"]
```

Run mailpeter with: `mailpeter -l 127.0.0.1:8989`

## Send Mail

Post content should look like:

```JSON
{
  "mail": "user@mail.com",
  "subject": "my subject",
  "text": "<html><strong>Hello</strong>, we support html and text mails :-)</html>"
}
```
Post request to: `http://127.0.0.1:8989/mail/contact/`

#### Send with attachment

```BASH
curl -i -X PUT -H "Content-Type: multipart/form-data" \
  -F mail=me@example.org \
  -F subject="my subject" \
  -F text="Have you seen this files?" \
  -F "file=@/home/user/Documents/my-contract.pdf" \
  http://127.0.0.1:8989/mail/contact/
```

## Run from CLI

Mail sending from Command line is supported, text can come from STDIN or from `--text` parameter.

Other options are:

```
Options:
  -A, --attachment [<ATTACHMENT>...]  Path to attachment file
  -c, --config <CONFIG>               Path to config
  -l, --listen <LISTEN>               Listen on IP:PORT, like: 127.0.0.1:8989
  -s, --subject <SUBJECT>             Mail subject for command line usage
  -t, --text <TEXT>                   Mail text for command line usage, stdin without -t work too
  -h, --help                          Print help
  -V, --version                       Print version
```

## Use as System Mailer

mailpeter can be used for systemmail for example for cron messages. To act as a sendmail replacement add symlink like:

```
ln -s /usr/bin/mailpeter /usr/sbin/sendmail
```

`alias` in config needs a mail address for that.
