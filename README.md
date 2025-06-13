# Installation Guide (Windows)
1. On the right side, click on **Releases**.
2. Download the latest release exe file.

# Installation Guide (other platforms)
1. Open a command prompt and navigate to some temporary folder.
2. Clone this repository: `git clone https://github.com/BioTomateDE/DiscordMassRedact`
3. Navigate into the cloned repository: `cd ./DiscordMassRedact`.
4. Install [Cargo](https://www.rust-lang.org/tools/install) if you haven't already.
5. Build the program: `cargo b -r`.
6. The built program binary is now located in `./target/release/discord-selfbot-delete-all`.

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
13. Run the program by typing `discord-selfbot-delete-all` on Windows or `./discord-selfbot-delete-all` on other platforms.
14. The program should function properly now. It might take a while to edit all messages because Discord has a slow rate limit for editing old messages (probably to prevent mass redacts like this).

# Licence
Permission is hereby granted to any individual to install, use, and modify this software for **personal, non-commercial purposes**, subject to the following conditions:

1. **Non-Commercial Use Only**: The software may not be used for commercial purposes without prior written permission from the creator. Commercial use includes, but is not limited to, selling the software, using it as part of a paid product or service, or using it to generate revenue.
2. **Prohibited Use**: The software may not be used for any unlawful or harmful activity. Users are solely responsible for compliance with all applicable laws in their jurisdiction.
3. **Source Reuse**: You may reuse parts of the source code in your own projects only if those projects are:
   - Open source and
   - Freely available (without any payment or licensing fee).
   Exceptions require written permission from the creator.
4. **No Warranty**: This software is provided “as is,” without warranty of any kind. The author is not liable for any damages arising from the use of this software.
5. **Governing Law**: This license is governed by the laws of Germany.
6. The creator of this program (BioTomateDE) is not in any way responsible for termination or other punishments of your Discord account as a result of using this selfbot program.
   Selfbots are against Discord's Terms of Service; Hence you are running this application at your own risk.
   However, your account is unlikely to get banned since this implementation respects Discord's `retry_after` cooldowns and uses a fake browser User Agent making it impossible to *prove* that you are using a selfbot.

For commercial licensing or exceptions, contact: [latuskati+eula@gmail.com]
