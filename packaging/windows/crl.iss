#define MyAppName "Codex-Resume-Loop"
#define MyAppVersion GetEnv("CRL_VERSION")
#define MyAppPublisher "noooob-coder"
#define MyAppURL "https://github.com/noooob-coder/Codex-Resume-Loop"
#define MyAppExeName "crl-desktop.exe"
#define MyCliExeName "crl.exe"
#define MyOutputDir GetEnv("CRL_OUTPUT_DIR")
#define MySourceDir GetEnv("CRL_STAGE_DIR")

[Setup]
AppId={{C7E0705D-8C84-4809-A73E-D0E52D8D2F5C}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={autopf}\Codex-Resume-Loop
DefaultGroupName=Codex-Resume-Loop
DisableProgramGroupPage=no
OutputDir={#MyOutputDir}
OutputBaseFilename=crl-setup-windows-x64-{#MyAppVersion}
Compression=lzma
SolidCompression=yes
ArchitecturesInstallIn64BitMode=x64compatible
WizardStyle=modern
SetupIconFile={#MySourceDir}\crl-icon.ico
UninstallDisplayIcon={app}\{#MyAppExeName}
ChangesEnvironment=yes
PrivilegesRequired=lowest

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "Create desktop shortcut"; GroupDescription: "Additional tasks:"; Flags: unchecked
Name: "addtopath"; Description: "Add Codex-Resume-Loop CLI to PATH"; GroupDescription: "Additional tasks:"; Flags: checkedonce

[Files]
Source: "{#MySourceDir}\crl-desktop.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#MySourceDir}\crl.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#MySourceDir}\README.txt"; DestDir: "{app}"; DestName: "README.txt"; Flags: ignoreversion

[Icons]
Name: "{group}\Codex-Resume-Loop Desktop"; Filename: "{app}\{#MyAppExeName}"
Name: "{group}\README"; Filename: "{app}\README.txt"
Name: "{group}\Uninstall Codex-Resume-Loop"; Filename: "{uninstallexe}"
Name: "{autodesktop}\Codex-Resume-Loop Desktop"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Registry]
Root: HKCU; Subkey: "Environment"; ValueType: expandsz; ValueName: "Path"; ValueData: "{olddata};{app}"; Check: NeedsAddPath(ExpandConstant('{app}')); Tasks: addtopath; Flags: preservestringtype

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "Launch Codex-Resume-Loop Desktop"; Flags: nowait postinstall skipifsilent

[Code]
function NeedsAddPath(Param: string): boolean;
var
  OrigPath: string;
begin
  if not RegQueryStringValue(HKCU, 'Environment', 'Path', OrigPath) then
    OrigPath := '';
  Result := Pos(';' + Uppercase(Param) + ';', ';' + Uppercase(OrigPath) + ';') = 0;
end;
