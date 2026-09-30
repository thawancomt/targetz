; Targetz NSIS Installer Script
; Can be compiled with makensis on Windows or Linux

Unicode True
!include "MUI2.nsh"

Name "Targetz"
OutFile "..\dist\Targetz-Setup-x86_64.exe"
InstallDir "$LOCALAPPDATA\Programs\Targetz"
InstallDirRegKey HKCU "Software\Targetz" "Install_Dir"
RequestExecutionLevel user

!define MUI_ICON "..\assets\icon.ico"
!define MUI_UNICON "..\assets\icon.ico"

!define MUI_ABORTWARNING

; Pages
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!define MUI_FINISHPAGE_RUN "$INSTDIR\targetz.exe"
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "English"

Section "Targetz Core" SecCore
    SetOutPath "$INSTDIR"
    File "..\target\x86_64-pc-windows-msvc\release\targetz.exe"
    
    ; Uninstaller
    WriteUninstaller "$INSTDIR\Uninstall.exe"
    
    ; Registry for uninstaller & Add/Remove Programs
    WriteRegStr HKCU "Software\Targetz" "Install_Dir" "$INSTDIR"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Targetz" "DisplayName" "Targetz"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Targetz" "UninstallString" '"$INSTDIR\Uninstall.exe"'
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Targetz" "DisplayIcon" '"$INSTDIR\targetz.exe"'
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Targetz" "DisplayVersion" "0.1.0"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Targetz" "Publisher" "Whatever"
    
    ; Shortcuts
    CreateDirectory "$SMPROGRAMS\Targetz"
    CreateShortcut "$SMPROGRAMS\Targetz\Targetz.lnk" "$INSTDIR\targetz.exe"
    CreateShortcut "$SMPROGRAMS\Targetz\Uninstall.lnk" "$INSTDIR\Uninstall.exe"
    CreateShortcut "$DESKTOP\Targetz.lnk" "$INSTDIR\targetz.exe"
SectionEnd

Section "Uninstall"
    Delete "$DESKTOP\Targetz.lnk"
    Delete "$SMPROGRAMS\Targetz\Targetz.lnk"
    Delete "$SMPROGRAMS\Targetz\Uninstall.lnk"
    RMDir "$SMPROGRAMS\Targetz"
    
    Delete "$INSTDIR\targetz.exe"
    Delete "$INSTDIR\Uninstall.exe"
    RMDir "$INSTDIR"
    
    DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Targetz"
    DeleteRegKey HKCU "Software\Targetz"
SectionEnd
