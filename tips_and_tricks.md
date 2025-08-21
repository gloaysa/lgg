# Tips and tricks

I'm stealing the idea of [tips and tricks from jrnl](https://jrnl.sh/en/stable/tips-and-tricks/ "jrnl tips and tricks") too. Check it out, some of them works with lgg with some modifications.

## iOS Shortcut to quickly create entries from your phone:

Download and install this [shortcut](https://www.icloud.com/shortcuts/7fd8cdbbb7bb44038577c953388d593f "iOS Shorcut") to create formatted entries in the folder of your choosing. Works really well if you have your journal_path set to a folder in iCloud or similar service.

## macOS: Launch `lgg` from outside the terminal

With the following steps, you can open lgg when you are not in the terminal to input a new entry. The result is the same as running `lgg` without anything else in your terminal: it will open a buffer where you can write, and upon saving and closing, it will add the new entry. Only that in this case, the new tab/window will be automatically closed.

> [!IMPORTANT] The following script assumes you have Wezterm Terminal installed!

You can create a new app that will open Wezterm running `lgg` following this steps:

1. Open Automator and create a new Application.
2. Paste the following script:

```applescript
do shell script "
if /opt/homebrew/bin/wezterm cli list >/dev/null 2>&1; then
  /opt/homebrew/bin/wezterm cli spawn -- /bin/zsh -lc 'lgg'
else
  /opt/homebrew/bin/wezterm start -- /bin/zsh -lc 'lgg'
fi >/dev/null 2>&1 &
"

delay 0.1
tell application "Wezterm" to activate
```

3. Save the new application in your Applications directory with the name `lgg`.
4. You can now do `cmd space` and search for lgg to open it.

> [!NOTE] If you are using Raycast, go to settings/Extensions/Applications and search for lgg. You can set an alias or even a hotkey!

---
