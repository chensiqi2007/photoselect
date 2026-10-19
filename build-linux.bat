@echo off
title PhotoSelect Desktop - Linux Packager
echo Starting Linux build in WSL Ubuntu...
wsl.exe -d Ubuntu -e bash -c "cd $(wslpath '%~dp0') && sed -i 's/\r$//' ./build-linux.sh && chmod +x ./build-linux.sh && ./build-linux.sh"
pause
