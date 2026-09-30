; Targetz Inno Setup Script
; Generates Targetz-Setup-x86_64.exe

[Setup]
AppName=Targetz
AppVersion=0.1.0
AppPublisher=Whatever
DefaultDirName={localappdata}\Programs\Targetz
DefaultGroupName=Targetz
UninstallDisplayIcon={app}\targetz.exe
Compression=lzma2/ultra64
SolidCompression=yes
OutputDir=..\dist
OutputBaseFilename=Targetz-Setup-x86_64
SetupIconFile=..\assets\icon.ico
PrivilegesRequired=lowest
DisableWelcomePage=no
DisableDirPage=auto
DisableProgramGroupPage=auto

[Files]
Source: "..\target\x86_64-pc-windows-msvc\release\targetz.exe"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\Targetz"; Filename: "{app}\targetz.exe"
Name: "{autodesktop}\Targetz"; Filename: "{app}\targetz.exe"; Tasks: desktopicon

[Tasks]
Name: "desktopicon"; Description: "Create a &desktop shortcut"; GroupDescription: "Additional icons:"

[Run]
Filename: "{app}\targetz.exe"; Description: "{cm:LaunchProgram,Targetz}"; Flags: nowait postinstall skipifsilent
