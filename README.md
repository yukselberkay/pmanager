<h1 align="center">
Pmanager
</h1>
<div align="center">

![GitHub](https://img.shields.io/github/license/yukselberkay/pmanager?style=for-the-badge)
![release](https://img.shields.io/badge/version-0.9.7-orange?style=for-the-badge)

<img src="images/logo.png" width="300">

</div>


## Demo
![demo](/images/demo.gif)

## Description
Store and retrieve your passwords from a secure offline database. Check if your passwords has leaked previously to prevent targeted password reuse attacks.

## Why develop another password manager ?
- This project was initially born from my desire to learn Rust.
- I was tired of using the clunky GUI of keepassxc.
- I wanted to learn more about cryptography.
- For fun. :)

## Features
- Secure password storage with state of the art cryptographic algorithms.
  -  Multiple iterations of argon2id for key derivation to make it harder for attacker to conduct brute force attacks.
  -  Aes-gcm256 for database encryption. 
- Custom encrypted key-value database which ensures data integrity.(Read the blog post I wrote about it [here](https://yukselberkay.github.io/programming/2022/09/12/post-keyval-db.html).)
- Easy to install and to use. Does not require connection to an external service for its core functionality.
- Check if your passwords are leaked before to avoid targeted password reuse attacks.
  - This works by hashing your password with keccak-512 and sending the first 10 digits to [XposedOrNot](https://xposedornot.com/api_doc).

## Installation
Pmanager depends on "pkg-config" and "libssl-dev" packages on **ubuntu**. Simply install them with
```bash
sudo apt install pkg-config libssl-dev -y
```

Download the binary file according to your current OS from [releases](https://github.com/yukselberkay/pmanager/releases), and add the binary location to PATH environment variable and you are good to go.


## Building from source
## Ubuntu & WSL
```bash
sudo apt update -y && sudo apt install curl 
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
sudo apt install build-essential -y
sudo apt install pkg-config libssl-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev git -y
git clone https://github.com/yukselberkay/pmanager
cd pmanager
make install

```

## Windows
- Follow the instructions here to install Rust. -> https://www.rust-lang.org/tools/install
- Then install git for Windows. -> https://gitforwindows.org/
```powershell
git clone https://github.com/yukselberkay/pmanager
cd pmanager
cargo build --release
```

## Mac
```bash
git clone https://github.com/yukselberkay/pmanager
cd pmanager
cargo build --release
```

## Documentation
**Firstly the database needs to be initialized using "init" command.**
### Init
```bash
# Initializes the database in the home directory.
pmanager init --db-path ~
```
### Insert
```bash
# Insert a new user and password pair to the database.
pmanager insert --domain github.com
```

### Get
```bash
# Get a specific record by domain. This command will first copy username and then password to clipboard.
pmanager get --domain github.com
```
### List
```bash
# List every record in the database.
pmanager list
```
### Update
```bash
# Update a record by domain.
pmanager update --domain github.com
```
### Delete
```bash
# Deletes a record associated with domain from the database.
pmanager delete github.com
```
### Leaked
```bash
# Check if a password in your database is leaked before.
pmanager leaked --domain github.com
```

```
pmanager 0.9.7

USAGE:
    pmanager [OPTIONS] [SUBCOMMAND]

OPTIONS:
    -d, --debug      
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    clip      Copy password to clipboard by domain
    delete    Delete a key value pair from database
    get       Copy username and then password to clipboard by domain
    help      Print this message or the help of the given subcommand(s)
    init      Initialize pmanager
    insert    Insert a user password pair associated with a domain to database
    leaked    Check if a password associated with your domain is leaked
    list      Lists every record in the database
    update    Update a record from database
```
## Support
Bitcoin Address -> bc1qrmcmgasuz78d0g09rllh9upurnjwzpn07vmmyj
