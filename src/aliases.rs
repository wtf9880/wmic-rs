#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Alias {
    pub name: &'static str,
    pub class: &'static str,
    pub description: &'static str,
    pub namespace: &'static str,
}

macro_rules! aliases {
    ($(($name:literal, $class:literal, $description:literal $(, $namespace:literal)?)),+ $(,)?) => {
        pub const ALIASES: &[Alias] = &[
            $(Alias {
                name: $name,
                class: $class,
                description: $description,
                namespace: aliases!(@namespace $( $namespace )?),
            }),+
        ];
    };
    (@namespace $namespace:literal) => { $namespace };
    (@namespace) => { r"root\cimv2" };
}

// The names and descriptions are the inbox English (en-US) role. Class mappings are kept
// separately from parsing so an installed ROOT\Cli role can replace this table later.
aliases! {
    ("ALIAS", "MSFT_CliAlias", "Access to the aliases available on the local system", r"root\cli"),
    ("BASEBOARD", "Win32_BaseBoard", "Base board (also known as a motherboard or system board) management."),
    ("BIOS", "Win32_BIOS", "Basic input/output services (BIOS) management."),
    ("BOOTCONFIG", "Win32_BootConfiguration", "Boot configuration management."),
    ("CDROM", "Win32_CDROMDrive", "CD-ROM management."),
    ("COMPUTERSYSTEM", "Win32_ComputerSystem", "Computer system management."),
    ("CPU", "Win32_Processor", "CPU management."),
    ("CSPRODUCT", "Win32_ComputerSystemProduct", "Computer system product information from SMBIOS."),
    ("DATAFILE", "CIM_DataFile", "DataFile Management."),
    ("DCOMAPP", "Win32_DCOMApplication", "DCOM Application management."),
    ("DESKTOP", "Win32_Desktop", "User's Desktop management."),
    ("DESKTOPMONITOR", "Win32_DesktopMonitor", "Desktop Monitor management."),
    ("DEVICEMEMORYADDRESS", "Win32_DeviceMemoryAddress", "Device memory addresses management."),
    ("DISKDRIVE", "Win32_DiskDrive", "Physical disk drive management."),
    ("DISKQUOTA", "Win32_DiskQuota", "Disk space usage for NTFS volumes."),
    ("DMACHANNEL", "Win32_DMAChannel", "Direct memory access (DMA) channel management."),
    ("ENVIRONMENT", "Win32_Environment", "System environment settings management."),
    ("FSDIR", "Win32_Directory", "Filesystem directory entry management."),
    ("GROUP", "Win32_Group", "Group account management."),
    ("IDECONTROLLER", "Win32_IDEController", "IDE Controller management."),
    ("IRQ", "Win32_IRQResource", "Interrupt request line (IRQ) management."),
    ("JOB", "Win32_ScheduledJob", "Provides access to jobs scheduled using the schedule service."),
    ("LOADORDER", "Win32_LoadOrderGroup", "Management of system services that define execution dependencies."),
    ("LOGICALDISK", "Win32_LogicalDisk", "Local storage device management."),
    ("LOGON", "Win32_LogonSession", "LOGON Sessions."),
    ("MEMCACHE", "Win32_CacheMemory", "Cache memory management."),
    ("MEMORYCHIP", "Win32_PhysicalMemory", "Memory chip information."),
    ("MEMPHYSICAL", "Win32_PhysicalMemoryArray", "Computer system's physical memory management."),
    ("NETCLIENT", "Win32_NetworkClient", "Network Client management."),
    ("NETLOGIN", "Win32_NetworkLoginProfile", "Network login information (of a particular user) management."),
    ("NETPROTOCOL", "Win32_NetworkProtocol", "Protocols (and their network characteristics) management."),
    ("NETUSE", "Win32_NetworkConnection", "Active network connection management."),
    ("NIC", "Win32_NetworkAdapter", "Network Interface Controller (NIC) management."),
    ("NICCONFIG", "Win32_NetworkAdapterConfiguration", "Network adapter management."),
    ("NTDOMAIN", "Win32_NTDomain", "NT Domain management."),
    ("NTEVENT", "Win32_NTLogEvent", "Entries in the NT Event Log."),
    ("NTEVENTLOG", "Win32_NTEventlogFile", "NT eventlog file management."),
    ("ONBOARDDEVICE", "Win32_OnBoardDevice", "Management of common adapter devices built into the motherboard (system board)."),
    ("OS", "Win32_OperatingSystem", "Installed Operating System/s management."),
    ("PAGEFILE", "Win32_PageFileUsage", "Virtual memory file swapping management."),
    ("PAGEFILESET", "Win32_PageFileSetting", "Page file settings management."),
    ("PARTITION", "Win32_DiskPartition", "Management of partitioned areas of a physical disk."),
    ("PORT", "Win32_PortResource", "I/O port management."),
    ("PORTCONNECTOR", "Win32_PortConnector", "Physical connection ports management."),
    ("PRINTER", "Win32_Printer", "Printer device management."),
    ("PRINTERCONFIG", "Win32_PrinterConfiguration", "Printer device configuration management."),
    ("PRINTJOB", "Win32_PrintJob", "Print job management."),
    ("PROCESS", "Win32_Process", "Process management."),
    ("PRODUCT", "Win32_Product", "Installation package task management."),
    ("QFE", "Win32_QuickFixEngineering", "Quick Fix Engineering."),
    ("QUOTASETTING", "Win32_QuotaSetting", "Setting information for disk quotas on a volume."),
    ("RDACCOUNT", "Win32_TSAccount", "Remote Desktop connection permission management.", r"root\cimv2\terminalservices"),
    ("RDNIC", "Win32_TSNetworkAdapterSetting", "Remote Desktop connection management on a specific network adapter.", r"root\cimv2\terminalservices"),
    ("RDPERMISSIONS", "Win32_TSPermissionsSetting", "Permissions to a specific Remote Desktop connection.", r"root\cimv2\terminalservices"),
    ("RDTOGGLE", "Win32_TerminalServiceSetting", "Turning Remote Desktop listener on or off remotely.", r"root\cimv2\terminalservices"),
    ("RECOVEROS", "Win32_OSRecoveryConfiguration", "Information that will be gathered from memory when the operating system fails."),
    ("REGISTRY", "StdRegProv", "Computer system registry management.", r"root\default"),
    ("SCSICONTROLLER", "Win32_SCSIController", "SCSI Controller management."),
    ("SERVER", "Win32_PerfRawData_PerfNet_Server", "Server information management."),
    ("SERVICE", "Win32_Service", "Service application management."),
    ("SHADOWCOPY", "Win32_ShadowCopy", "Shadow copy management."),
    ("SHADOWSTORAGE", "Win32_ShadowStorage", "Shadow copy storage area management."),
    ("SHARE", "Win32_Share", "Shared resource management."),
    ("SOFTWAREELEMENT", "Win32_SoftwareElement", "Management of the elements of a software product installed on a system."),
    ("SOFTWAREFEATURE", "Win32_SoftwareFeature", "Management of software product subsets of SoftwareElement."),
    ("SOUNDDEV", "Win32_SoundDevice", "Sound Device management."),
    ("STARTUP", "Win32_StartupCommand", "Management of commands that run automatically when users log onto the computer system."),
    ("SYSACCOUNT", "Win32_SystemAccount", "System account management."),
    ("SYSDRIVER", "Win32_SystemDriver", "Management of the system driver for a base service."),
    ("SYSTEMENCLOSURE", "Win32_SystemEnclosure", "Physical system enclosure management."),
    ("SYSTEMSLOT", "Win32_SystemSlot", "Management of physical connection points including ports, slots and peripherals, and proprietary connections points."),
    ("TAPEDRIVE", "Win32_TapeDrive", "Tape drive management."),
    ("TEMPERATURE", "Win32_TemperatureProbe", "Data management of a temperature sensor (electronic thermometer)."),
    ("TIMEZONE", "Win32_TimeZone", "Time zone data management."),
    ("UPS", "Win32_UninterruptiblePowerSupply", "Uninterruptible power supply (UPS) management."),
    ("USERACCOUNT", "Win32_UserAccount", "User account management."),
    ("VOLTAGE", "Win32_VoltageProbe", "Voltage sensor (electronic voltmeter) data management."),
    ("VOLUME", "Win32_Volume", "Local storage volume management."),
    ("VOLUMEQUOTASETTING", "Win32_VolumeQuotaSetting", "Associates the disk quota setting with a specific disk volume."),
    ("VOLUMEUSERQUOTA", "Win32_VolumeUserQuota", "Per user storage volume quota management."),
    ("WMISET", "Win32_WMISetting", "WMI service operational parameters management."),
}

pub fn find(name: &str) -> Option<&'static Alias> {
    ALIASES.iter().find(|alias| alias.name.eq_ignore_ascii_case(name))
}

pub fn global_help() -> String {
    let mut text = String::from(
        "WMIC is deprecated.\n\n[global switches] <command>\n\n\
The following global switches are available:\n\
/NAMESPACE           Path for the namespace the alias operate against.\n\
/ROLE                Path for the role containing the alias definitions.\n\
/NODE                Servers the alias will operate against.\n\
/IMPLEVEL            Client impersonation level.\n\
/AUTHLEVEL           Client authentication level.\n\
/LOCALE              Language id the client should use.\n\
/PRIVILEGES          Enable or disable all privileges.\n\
/TRACE               Outputs debugging information to stderr.\n\
/RECORD              Logs all input commands and output.\n\
/INTERACTIVE         Sets or resets the interactive mode.\n\
/FAILFAST            Sets or resets the FailFast mode.\n\
/USER                User to be used during the session.\n\
/PASSWORD            Password to be used for session login.\n\
/OUTPUT              Specifies the mode for output redirection.\n\
/APPEND              Specifies the mode for output redirection.\n\
/AGGREGATE           Sets or resets aggregate mode.\n\
/AUTHORITY           Specifies the <authority type> for the connection.\n\
/?[:<BRIEF|FULL>]    Usage information.\n\n\
For more information on a specific global switch, type: switch-name /?\n\n\n\
The following alias/es are available in the current role:\n",
    );
    for alias in ALIASES {
        text.push_str(&format!("{:<25}- {}\n", alias.name, alias.description));
    }
    text.push_str(
        "\nFor more information on a specific alias, type: alias /?\n\n\
CLASS     - Escapes to full WMI schema.\n\
PATH      - Escapes to full WMI object paths.\n\
CONTEXT   - Displays the state of all the global switches.\n\
QUIT/EXIT - Exits the program.\n\n\
For more information on CLASS/PATH/CONTEXT, type: (CLASS | PATH | CONTEXT) /?\n",
    );
    text
}

pub fn alias_help(alias: &Alias) -> String {
    format!(
        "{0} - {1}\n\nHint: BNF for Alias usage.\n{0} [WHERE <condition>] [LIST [BRIEF|FULL|INSTANCE|STATUS|SYSTEM] | GET <properties> [/VALUE|/ALL] | CALL <method> | SET <assignments> | CREATE <assignments> | DELETE] [/FORMAT:<format>]\n\nUnderlying WMI class: {2}\nNamespace: {3}\n",
        alias.name, alias.description, alias.class, alias.namespace
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn inbox_role_has_expected_alias_count_and_unique_names() {
        assert_eq!(ALIASES.len(), 81);
        let unique: HashSet<_> = ALIASES.iter().map(|a| a.name).collect();
        assert_eq!(unique.len(), ALIASES.len());
    }

    #[test]
    fn lookup_is_ascii_case_insensitive() {
        assert_eq!(find("pRoCeSs").unwrap().class, "Win32_Process");
    }

    #[test]
    fn every_mapping_has_a_valid_wmi_identifier_and_namespace() {
        for alias in ALIASES {
            assert!(alias.class.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'));
            assert!(alias.namespace.starts_with("root\\"));
            assert!(!alias.description.is_empty());
        }
    }

    #[test]
    fn global_help_mentions_every_alias_once() {
        let help = global_help();
        for alias in ALIASES {
            assert_eq!(help.matches(&format!("{:<25}-", alias.name)).count(), 1);
        }
    }
}
