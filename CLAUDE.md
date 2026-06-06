# Project CLAUDE.md

## Windows Directory Links (Git Bash)

Use PowerShell `Junction` — no admin rights required:

```bash
powershell -Command "New-Item -ItemType Junction -Path 'C:\path\to\link' -Target 'C:\path\to\target'"
```

Verify: `cmd.exe //c "dir /AL C:\path\to\parent"`

Never use `mklink` from Git Bash or `New-Item -ItemType SymbolicLink` (requires admin).
