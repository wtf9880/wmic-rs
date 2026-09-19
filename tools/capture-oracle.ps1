param([string]$OutputDirectory = "$PWD\wmic-oracle")

$ErrorActionPreference = 'Stop'
$wmic = "$env:WINDIR\System32\wbem\wmic.exe"
if (-not (Test-Path $wmic)) { throw "Legacy WMIC is not installed on this runner" }
New-Item -ItemType Directory -Force $OutputDirectory | Out-Null

$aliases = @(
    'ALIAS','BASEBOARD','BIOS','BOOTCONFIG','CDROM','COMPUTERSYSTEM','CPU','CSPRODUCT',
    'DATAFILE','DCOMAPP','DESKTOP','DESKTOPMONITOR','DEVICEMEMORYADDRESS','DISKDRIVE',
    'DISKQUOTA','DMACHANNEL','ENVIRONMENT','FSDIR','GROUP','IDECONTROLLER','IRQ','JOB',
    'LOADORDER','LOGICALDISK','LOGON','MEMCACHE','MEMORYCHIP','MEMPHYSICAL','NETCLIENT',
    'NETLOGIN','NETPROTOCOL','NETUSE','NIC','NICCONFIG','NTDOMAIN','NTEVENT','NTEVENTLOG',
    'ONBOARDDEVICE','OS','PAGEFILE','PAGEFILESET','PARTITION','PORT','PORTCONNECTOR',
    'PRINTER','PRINTERCONFIG','PRINTJOB','PROCESS','PRODUCT','QFE','QUOTASETTING',
    'RDACCOUNT','RDNIC','RDPERMISSIONS','RDTOGGLE','RECOVEROS','REGISTRY','SCSICONTROLLER',
    'SERVER','SERVICE','SHADOWCOPY','SHADOWSTORAGE','SHARE','SOFTWAREELEMENT',
    'SOFTWAREFEATURE','SOUNDDEV','STARTUP','SYSACCOUNT','SYSDRIVER','SYSTEMENCLOSURE',
    'SYSTEMSLOT','TAPEDRIVE','TEMPERATURE','TIMEZONE','UPS','USERACCOUNT','VOLTAGE',
    'VOLUME','VOLUMEQUOTASETTING','VOLUMEUSERQUOTA','WMISET'
)

@{
    capturedAt = (Get-Date).ToUniversalTime().ToString('o')
    os = [Environment]::OSVersion.VersionString
    culture = [Globalization.CultureInfo]::CurrentCulture.Name
    uiCulture = [Globalization.CultureInfo]::CurrentUICulture.Name
    wmicSha256 = (Get-FileHash $wmic -Algorithm SHA256).Hash
    wmicVersion = (Get-Item $wmic).VersionInfo.FileVersion
} | ConvertTo-Json | Set-Content -Encoding utf8 "$OutputDirectory\metadata.json"

& $wmic '/?' 2>&1 | Set-Content -Encoding utf8 "$OutputDirectory\global-help.txt"
foreach ($alias in $aliases) {
    & $wmic $alias '/?' 2>&1 | Set-Content -Encoding utf8 "$OutputDirectory\alias-$($alias.ToLowerInvariant()).txt"
    foreach ($verb in @('get', 'list', 'call')) {
        & $wmic $alias $verb '/?' 2>&1 |
            Set-Content -Encoding utf8 "$OutputDirectory\alias-$($alias.ToLowerInvariant())-$verb.txt"
    }
}

try {
    Get-CimInstance -Namespace root/cli -ClassName MSFT_CliAlias -ErrorAction Stop |
        Select-Object * -ExcludeProperty CimClass,CimInstanceProperties,CimSystemProperties |
        ConvertTo-Json -Depth 8 |
        Set-Content -Encoding utf8 "$OutputDirectory\root-cli-aliases.json"
} catch {
    $_ | Out-String | Set-Content -Encoding utf8 "$OutputDirectory\root-cli-error.txt"
}

Get-ChildItem "$env:WINDIR\System32\wbem" -File |
    Where-Object Extension -In '.xsl','.mof','.mfl' |
    Select-Object Name,Length,@{n='SHA256';e={(Get-FileHash $_.FullName -Algorithm SHA256).Hash}} |
    ConvertTo-Json |
    Set-Content -Encoding utf8 "$OutputDirectory\wbem-files.json"

Write-Host "Captured $($aliases.Count) aliases in $OutputDirectory"
