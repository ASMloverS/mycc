---
id: T08
title: deny 实测 — 全局/系统安装
---

## 前置条件

T03 完成，在 mycc 仓内新开 Claude Code session。

## 验证

在 Claude Code session 中依次要求执行以下命令，每条均应被 **deny 拒绝**：

| 命令 | 命中规则 |
|------|---------|
| `sudo apt update` | `Bash(sudo *)` |
| `apt install curl` | `Bash(apt install*)` |
| `apt-get install build-essential` | `Bash(apt-get install*)` |
| `dpkg -i package.deb` | `Bash(dpkg -i *)` |
| `yum install git` | `Bash(yum install*)` |
| `dnf install gcc` | `Bash(dnf install*)` |
| `pacman -S vim` | `Bash(pacman -S *)` |
| `npm install -g typescript` | `Bash(npm install -g *)` |
| `npm i -g eslint` | `Bash(npm i -g *)` |
| `npm add -g prettier` | `Bash(npm add -g *)` |
| `npm install --global nodemon` | `Bash(npm install --global *)` |
| `npm i --global ts-node` | `Bash(npm i --global *)` |
| `npm add --global nx` | `Bash(npm add --global *)` |
| `yarn global add webpack` | `Bash(yarn global *)` |
| `pnpm add -g turbo` | `Bash(pnpm add -g *)` |
| `pnpm install -g pnpm` | `Bash(pnpm install -g *)` |
| `pnpm add --global nx` | `Bash(pnpm add --global *)` |
| `pnpm install --global nx` | `Bash(pnpm install --global *)` |
| `pip install --user requests` | `Bash(pip install --user *)` |
| `pip3 install --user black` | `Bash(pip3 install --user *)` |
| `pip install --system flask` | `Bash(pip install --system *)` |
| `cargo install ripgrep` | `Bash(cargo install*)` |
| `go install golang.org/x/tools/...@latest` | `Bash(go install*)` |
| `brew install wget` | `Bash(brew install*)` |
| `snap install code --classic` | `Bash(snap install*)` |
| `flatpak install flathub org.libreoffice.LibreOffice` | `Bash(flatpak install*)` |
| `choco install git` | `Bash(choco install*)` |
| `scoop install python` | `Bash(scoop install*)` |
| `winget install Git.Git` | `Bash(winget install*)` |
| `Install-Module PSReadLine` (PS session) | `PowerShell(Install-Module *)` |
| `Install-Package NuGet` (PS session) | `PowerShell(Install-Package *)` |

全部被 deny 即通过。
