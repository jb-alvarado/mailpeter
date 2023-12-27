# mailpeter

Simple mailer and API for contact forms.

**Warning: This project is in a very early stage, do not use it in production!**

## Configuration

Config format is Toml, content of mailpeter.toml should look like:

```Toml
log_keep_count = 10
log_level = "info"
log_size_mb = 1
log_to_file = true
reverse_proxy_ip = "127.0.0.1"
limit_request_seconds = 30

[mail]
smtp = "smtp.example.org"
port = 587
user = "info@example.org"
password = "super-secure-mail-password"
starttls = true

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
