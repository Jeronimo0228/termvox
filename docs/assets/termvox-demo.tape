# TermVox 4K demo (~60s) — render: bash scripts/render-demo-4k.sh
Output docs/assets/termvox-demo-4k.mp4

Require termvox
Require bash

Set Shell bash
Set Theme "Catppuccin Mocha"
Set FontSize 44
Set Width 3840
Set Height 2160
Set Framerate 30
Set Padding 56
Set MarginFill "#141820"
Set BorderRadius 20
Set WindowBar Colorful
Set TypingSpeed 22ms

Type "echo 'TermVox — voice for terminal coding agents'"
Enter
Sleep 2s
Type "echo 'npm install -g termvox'"
Enter
Sleep 4s

Type "termvox --version"
Enter
Sleep 5s

Type "termvox doctor 2>&1 | head -24"
Enter
Sleep 12s

Type "termvox test --seconds 2 || true"
Enter
Sleep 14s

Type "echo"
Enter
Type "echo '# termvox shell --agent opencode'"
Enter
Sleep 2s
Type "echo '[mic bar] F8 / Ctrl+Space — push to talk'"
Enter
Sleep 3s
Type "echo 'Heard: Refactor auth to use structured errors'"
Enter
Sleep 3s
Type "echo 'Confirm [y] → prompt sent to agent TUI'"
Enter
Sleep 5s

Type "echo 'https://github.com/Jeronimo0228/termvox  |  alpha preview'"
Enter
Sleep 8s
