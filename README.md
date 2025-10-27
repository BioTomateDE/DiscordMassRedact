# Installation Guide (Arch btw)
Install the package from the AUR using your favourite AUR helper:
```console
yay -S discord-mass-redact
```

# Installation Guide (Other Linux Distributions, MacOS and Windows)
1. On the right side, click on **Releases** and find the version you'd like to install (presumably the latest one).
2. Download the corresponding archive for your platform (linux/mac/windows;x64/ARM).
3. Extract the archive and run the contained executable.

# Installation Guide (other platforms)
1. Open a command prompt and navigate to some temporary folder.
2. Clone this repository: `git clone https://github.com/BioTomateDE/DiscordMassRedact`
3. Navigate into the cloned repository: `cd ./DiscordMassRedact`.
4. Install [Rust Cargo](https://www.rust-lang.org/tools/install) if you haven't already.
5. Build the program: `cargo b -r`.
6. The built program binary is now located in `./target/release/discord-mass-redact`.

# Usage
1. In Discord, go to **Settings** → **Data & Privacy**.
2. Request to download all of your data.
3. After approximately 2 days (it may take longer), 
   you should get an email from Discord containing your data.
4. Download and extract the zip file from that email.
5. Obtain your discord token (just google "how to get my discord token").
6. Open a terminal and navigate to the folder where the downloaded/built program is located. (Skip this step if installed via AUR)
7. Run the program by typing 
   `./discord-mass-redact YOUR.DISCORDTOKEN C:/Users/YourUsername/Downloads/package/ some-deletion-mode`. (Remove the preceding `./` if installed via AUR)
   > You must replace the arguments with your discord token, 
   > the correct path to your discord data export and your desired mode respectively.
   > Type `./discord-mass-redact --help` for more information. (Remove the preceding `./` if installed via AUR)
8. The program should function properly now. 
   It might take a while to edit all messages because Discord has a slow rate limit 
   for editing old messages (probably to prevent mass redacts like this).

# Modes
There are different modes on how messages should be redacted.
You can choose one of the following:
- **Delete**: Deletes the entire message.
- **Shakespeare**: Overwrites the message's content to a quote from one of Shakespeare's works of similar length. 
- **Random words**: Replaces the message content with rubbish sentences.
*This is no longer available due to it sucking ass. 
If there is demand for this mode, I will add it back.*

# Options
There are some options you can choose from.
To list them, run the executable with the `--help` flag.

# Contributing
All contributions are welcome! Whether that's a pull request, a bug you found or a feature you wish for.

By contributing, you agree to:
- License your contributions under this project's license
- Certify you have the right to submit the code
- Allow the project maintainer to use your contributions
