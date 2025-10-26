# Installation Guide (Windows)
1. On the right side, click on **Releases**.
2. Download the latest release exe file.

# Installation Guide (other platforms)
1. Open a command prompt and navigate to some temporary folder.
2. Clone this repository: `git clone https://github.com/BioTomateDE/DiscordMassRedact`
3. Navigate into the cloned repository: `cd ./DiscordMassRedact`.
4. Install [Cargo](https://www.rust-lang.org/tools/install) if you haven't already.
5. Build the program: `cargo b -r`.
6. The built program binary is now located in `./target/release/discord-mass-redact`.

# Usage
1. In Discord, go to **Settings** → **Data & Privacy**.
2. Request to download all of your data.
3. After approximately 2 days, you should get an email from Discord containing your data.
4. Download the zip file from that email.
5. Extract the zip file.
6. Acquire your discord token (just google "how to get my discord token").
7. In the folder where you downloaded/built the program, create a file called `.env`.
8. Open the file in your favorite editor.
9. Add this line: `DISCORD_TOKEN="XXXXXXXXXXXXXXXXXXXXXXXX.XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"` (replace with your actual discord token).
10. Add another line: `DISCORD_MESSAGES_DIR="C:/Users/yourusername/Downloads/package/messages"` (replace with your actual path to the `messages` folder found inside your exported discord data).
11. Save the `.env` file.
12. Open a terminal and navigate to the folder where the downloaded/built program is located.
13. Run the program by typing `./discord-mass-redact`.
14. The program should function properly now. It might take a while to edit all messages because Discord has a slow rate limit for editing old messages (probably to prevent mass redacts like this).

# Licence
[GPL v3](https://www.gnu.org/licenses/gpl-3.0.en.html#license-text).