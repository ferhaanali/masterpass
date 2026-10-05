# 🛡️ Dataseal

Dataseal is a secure, lightweight encryption locker designed to keep your files and sensitive information safe.
Built with Rust, it combines strong cryptography with a minimal database engine, giving you both performance and protection.

Dataseal uses AES-256-GCM encryption with PBKDF2-HMAC-SHA256 key derivation, and introduces a unique design where your master password protects only the encryption keys, not the files directly. This makes it possible to change your password instantly without re-encrypting all data.

---

## ⚡ Features
- Strong cryptography (AES-256-GCM, PBKDF2-HMAC-SHA256)
- Lightweight and fast
- Minimal database engine for secure key storage
- Master password protects keys, not files, for instant password changes

---

💻 Installation

### Linux (bash) : 
```bash
# Download the latest release
curl -L https://raw.githubusercontent.com/Farhan-Ali-Rajpoot/dataseal/main/release/latest/download/dataseal-linux-x86_64-0.1.0.tar.gz -o dataseal.tar.gz

tar -xzf dataseal.tar.gz
sudo mv dataseal /usr/local/bin/
dataseal --help

```
### Windows (power_shell)  :
```powershell
mkdir $env:USERPROFILE\bin
curl -L https://raw.githubusercontent.com/Farhan-Ali-Rajpoot/dataseal/main/release/latest/download/dataseal-windows-x86_64-0.1.0.zip -o dataseal.zip
Expand-Archive dataseal.zip -DestinationPath $env:USERPROFILE\bin
[Environment]::SetEnvironmentVariable("PATH", $env:PATH + ";$env:USERPROFILE\bin", [EnvironmentVariableTarget]::User)
dataseal --help

```
