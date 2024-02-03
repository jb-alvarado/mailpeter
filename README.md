# mailpeter

Simple CLI mailer and HTTP API for contact forms.

**The project is still at an early stage, use it with caution!**

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
block_words = [
    "https?://",
    "selling",
    "holiday",
]                                          # Block mails with these words in the subject or mail body, regex is supported.

[[mail.recipients]]
allow_html = false                         # Send message as text (false), or allow html message.
direction = "contact"
mails = ["info@example.org", "office@example.org"]
send_copy = false

[[mail.recipients]]
allow_html = true
direction = "order"
mails = ["shop@example.org"]
send_copy = true                           # Send a copy from the message to the user.
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

## Spam protection

mailpeter can block messages based on keywords in subject or body. Add your words or regex to the `block_words` list in the mail section.
For example:

```TOML
block_words = [
    "https?://", # Block email with any http or https link.
    "\\@",       # Block email that contains email addresses.
    "selling",   # Block email that contains the word "selling".
    "holiday",   # Block email that contains the word "holiday".
]
```

## Run from CLI

Mail sending from Command line is supported, text can come from STDIN or from `--text` parameter.

Other options are:

```
Options:
  -A, --attachment [<ATTACHMENT>...]  Path to attachment file
  -c, --config <CONFIG>               Path to config
  -F, --full-name <FULL_NAME>         Set the sender full name, this override From header
  -l, --listen <LISTEN>               Listen on IP:PORT, like: 127.0.0.1:8989
  -L, --level <LEVEL>                 Log level, like: debug, info, warn, error, off
  -s, --subject <SUBJECT>             Mail subject for command line usage
      --message <MESSAGE>             Mail text for command line usage, stdin work too
  -h, --help                          Print help
  -V, --version                       Print version
```

## Use as System Mailer

mailpeter can be used for systemmail for example for cron messages. To act as a sendmail replacement add symlink like:

```
ln -s /usr/bin/mailpeter /usr/sbin/sendmail
```

`alias` in config needs a mail address for that.
