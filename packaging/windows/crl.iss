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
var
  RemoveHistoryOnUninstall: Boolean;

function NeedsAddPath(Param: string): boolean;
var
  OrigPath: string;
begin
  if not RegQueryStringValue(HKCU, 'Environment', 'Path', OrigPath) then
    OrigPath := '';
  Result := Pos(';' + Uppercase(Param) + ';', ';' + Uppercase(OrigPath) + ';') = 0;
end;

function NormalizeDir(Path: string): string;
begin
  Result := Trim(Path);
  if (Length(Result) >= 2) and (Result[1] = '"') and (Result[Length(Result)] = '"') then
    Result := Copy(Result, 2, Length(Result) - 2);
  StringChangeEx(Result, '/', '\', True);
  while (Length(Result) > 0) and (Result[Length(Result)] = '\') do
    Delete(Result, Length(Result), 1);
  Result := Uppercase(Result);
end;

function IsKnownLegacyInstallDir(Path: string): Boolean;
var
  Normalized: string;
begin
  Normalized := NormalizeDir(Path);
  Result :=
    (Normalized = NormalizeDir(ExpandConstant('{localappdata}') + '\Programs\CRL')) or
    (Normalized = NormalizeDir(GetEnv('LOCALAPPDATA') + '\Programs\CRL'));
end;

procedure DeleteIfPresent(Path: string);
begin
  if FileExists(Path) then
    DeleteFile(Path);
end;

procedure DeleteLegacyArtifactsInDir(Path: string);
var
  Dir: string;
begin
  Dir := AddBackslash(Path);
  DeleteIfPresent(Dir + 'crl.exe');
  DeleteIfPresent(Dir + 'codex-resume-loop.exe');
  DeleteIfPresent(Dir + 'crl.cmd');
  DeleteIfPresent(Dir + 'codex-resume-loop.cmd');
  DeleteIfPresent(Dir + 'crl.ps1');
  DeleteIfPresent(Dir + 'codex-resume-loop.ps1');
  DeleteIfPresent(Dir + 'crl');
  DeleteIfPresent(Dir + 'codex-resume-loop');
end;

function DirLooksLikeLegacyCrInstall(Path: string): Boolean;
var
  Expanded: string;
begin
  Expanded := AddBackslash(Path);
  Result :=
    IsKnownLegacyInstallDir(Path) or
    ((CompareText(ExtractFileName(NormalizeDir(Path)), 'CRL') = 0) and
      (FileExists(Expanded + 'crl.exe') or FileExists(Expanded + 'codex-resume-loop.exe'))) or
    ((CompareText(ExtractFileName(NormalizeDir(Path)), 'CODEX-RESUME-LOOP') = 0) and
      (FileExists(Expanded + 'crl.exe') or FileExists(Expanded + 'codex-resume-loop.exe')));
end;

function PruneLegacyPathEntries(PathValue: string; CurrentApp: string): string;
var
  Remaining: string;
  Item: string;
  Delimiter: Integer;
  Cleaned: string;
  Normalized: string;
begin
  Remaining := PathValue;
  Cleaned := '';
  while Remaining <> '' do
  begin
    Delimiter := Pos(';', Remaining);
    if Delimiter = 0 then
    begin
      Item := Remaining;
      Remaining := '';
    end
    else
    begin
      Item := Copy(Remaining, 1, Delimiter - 1);
      Delete(Remaining, 1, Delimiter);
    end;

    Normalized := NormalizeDir(Item);
    if (Normalized = '') then
      continue;
    if Normalized = NormalizeDir(CurrentApp) then
    begin
      if Cleaned <> '' then
        Cleaned := Cleaned + ';';
      Cleaned := Cleaned + Item;
      continue;
    end;
    if DirLooksLikeLegacyCrInstall(Item) then
      continue;

    if Cleaned <> '' then
      Cleaned := Cleaned + ';';
    Cleaned := Cleaned + Item;
  end;
  Result := Cleaned;
end;

procedure VisitCandidateDir(Path: string; CurrentApp: string; var Seen: string);
var
  Normalized: string;
begin
  Normalized := NormalizeDir(Path);
  if (Normalized = '') or (Normalized = NormalizeDir(CurrentApp)) then
    exit;
  if Pos(';' + Normalized + ';', Seen) > 0 then
    exit;

  Seen := Seen + Normalized + ';';
  DeleteLegacyArtifactsInDir(Path);

  if IsKnownLegacyInstallDir(Path) and DirExists(Path) then
    DelTree(Path, True, True, True);
end;

procedure VisitPathList(PathValue: string; CurrentApp: string; var Seen: string);
var
  Remaining: string;
  Item: string;
  Delimiter: Integer;
begin
  Remaining := PathValue;
  while Remaining <> '' do
  begin
    Delimiter := Pos(';', Remaining);
    if Delimiter = 0 then
    begin
      Item := Remaining;
      Remaining := '';
    end
    else
    begin
      Item := Copy(Remaining, 1, Delimiter - 1);
      Delete(Remaining, 1, Delimiter);
    end;

    VisitCandidateDir(Item, CurrentApp, Seen);
  end;
end;

procedure RemoveLegacyCliCopies();
var
  Seen: string;
  CurrentApp: string;
  UserPath: string;
begin
  CurrentApp := ExpandConstant('{app}');
  Seen := ';' + NormalizeDir(CurrentApp) + ';';

  VisitCandidateDir(GetEnv('USERPROFILE') + '\.local\bin', CurrentApp, Seen);
  VisitCandidateDir(GetEnv('LOCALAPPDATA') + '\Programs\CRL', CurrentApp, Seen);

  if RegQueryStringValue(HKCU, 'Environment', 'Path', UserPath) then
  begin
    VisitPathList(UserPath, CurrentApp, Seen);
    UserPath := PruneLegacyPathEntries(UserPath, CurrentApp);
    RegWriteExpandStringValue(HKCU, 'Environment', 'Path', UserPath);
  end;

  VisitPathList(GetEnv('PATH'), CurrentApp, Seen);
end;

procedure CurStepChanged(CurStep: TSetupStep);
begin
  if CurStep = ssPostInstall then
    RemoveLegacyCliCopies();
end;

function InitializeUninstall(): Boolean;
begin
  RemoveHistoryOnUninstall := False;
  Result := True;

  if UninstallSilent then
    exit;

  RemoveHistoryOnUninstall :=
    MsgBox(
      'Also remove local Codex-Resume-Loop state and history from this machine?',
      mbConfirmation,
      MB_YESNO or MB_DEFBUTTON2
    ) = IDYES;
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
var
  ConfigDir: string;
begin
  if (CurUninstallStep = usPostUninstall) and RemoveHistoryOnUninstall then
  begin
    ConfigDir := ExpandConstant('{userappdata}\shcem\crl-desktop\config');
    if DirExists(ConfigDir) then
      DelTree(ConfigDir, True, True, True);
  end;
end;
