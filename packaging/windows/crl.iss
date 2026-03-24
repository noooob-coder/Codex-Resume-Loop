#define MyAppName "CRL"
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
DefaultDirName={autopf}\CRL
DefaultGroupName=CRL
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
Name: "desktopicon"; Description: "创建桌面快捷方式"; GroupDescription: "附加任务："; Flags: unchecked
Name: "addtopath"; Description: "将 CRL CLI 添加到 PATH"; GroupDescription: "附加任务："; Flags: checkedonce

[Files]
Source: "{#MySourceDir}\crl-desktop.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#MySourceDir}\crl.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#MySourceDir}\README.txt"; DestDir: "{app}"; DestName: "README.txt"; Flags: ignoreversion

[Icons]
Name: "{group}\CRL Desktop"; Filename: "{app}\{#MyAppExeName}"
Name: "{group}\README"; Filename: "{app}\README.txt"
Name: "{autodesktop}\CRL Desktop"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Registry]
Root: HKCU; Subkey: "Environment"; ValueType: expandsz; ValueName: "Path"; ValueData: "{olddata};{app}"; Check: NeedsAddPath(ExpandConstant('{app}')); Tasks: addtopath; Flags: preservestringtype

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "启动 CRL Desktop"; Flags: nowait postinstall skipifsilent

[Code]
function NeedsAddPath(Param: string): boolean;
var
  OrigPath: string;
begin
  if not RegQueryStringValue(HKCU, 'Environment', 'Path', OrigPath) then
    OrigPath := '';
  Result := Pos(';' + Uppercase(Param) + ';', ';' + Uppercase(OrigPath) + ';') = 0;
end;
