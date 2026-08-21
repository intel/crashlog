# Lightweight Crash Log Framework (aka `iclg`)

[![CI](https://github.com/intel/crashlog/actions/workflows/ci.yml/badge.svg?event=push)](https://github.com/intel/crashlog/actions/workflows/ci.yml)

**[Download Binaries](https://github.com/intel/crashlog/releases/latest)** • **[Documentation](https://intel.github.io/crashlog/crates/intel_crashlog/)** • **[Getting Started](#getting-started)**

> **Experienced a crash on an Intel SoC? The hardware may have captured a Crash Log.**

Modern Intel SoCs automatically record hardware state during fatal crashes
(MCE, triple faults, unexpected resets) into on-die memory.
`iclg` extracts and decodes that binary data into human-readable JSON.

Check if your system has crash log data:

```console
$ iclg extract
```

## What can be done with the Crash Log?

Once extracted, you have several options depending on your needs:

- Use the `iclg` decoder provided in this repository to convert crash log
  records into structured JSON format for initial triage and automated
  processing.

  ```console
  $ iclg triage three_strike_timeout_with_xq.crashlog
  CORE_TIMEOUT.SINGLE_STUCK_TRANSACTION.13014002340H
  MCA.BANK3.INTERNAL_TIMER_ERROR.MSCOD_E184H
  RESET_CAUSE.GLOBAL_RESET.PMC_FW

  $ iclg decode three_strike_timeout_with_xq.crashlog |
  > jq '.crashlog_data.pcore.core0.thread0.thread.arch_state.mca.bank3'
  {
    "addr": "0xfffff80252753e75",
    "ctl": "0x7f",
    "misc": "0xfffff80252753e75",
    "status": "0xbe000000e1840400"
  }
  ```

- Send the Crash Log to your Intel representative for further support.

- For advanced analysis, the [Intel System Debugger](https://www.intel.com/content/www/us/en/developer/tools/oneapi/system-bring-up-toolkit.html)
  provides additional debugging capabilities (requires NDA).

## How does it work?

Intel® Crash Log Technology is a mechanism for collecting debug information
from the System-on-Chip (SoC), such as status/error registers and state
machines, and storing it in on-die memory. Data collection is triggered by
unrecoverable system errors, like fatal machine check exceptions or unexpected
resets.

When valid records are present in the SoC during boot, the BIOS copies them
into the ACPI Boot Error Record Table (BERT) to expose them to the OS. The data
stored in the BERT can then be accessed via Windows Event Logs, Linux sysfs, or
the EFI shell.

The Lightweight Crash Log Framework is a reference implementation designed for
decoding and extracting data using Intel® Crash Log Technology. It can function
as a standalone application or be integrated into other applications.

## Features Overview

- Extract Intel Crash Log records from Windows Event Logs, Linux sysfs, and the
  EFI shell.
- Store and convert the Intel Crash Log records in the UEFI CPER format
  (as described in the UEFI Specification Appendix N).
- Decode Intel Crash Log records and export the content as JSON.

>[!NOTE]
> This tool supports a limited number of platforms utilizing Crash Log
> technology. However, it can decode the common Crash Log header structure on
> unsupported platforms. We are actively working to expand platform support.
> Additionally, the set of registers collected may vary between projects and
> the register layout may be updated in future releases.

## Repository Structure

- **[Library](lib/):** A library for extracting and decoding Crash Log records.
- **[Collateral Tree](lib/collateral):** A collection of product-specific
  collateral needed for decoding Crash Log records.
- **[Command Line Tool](app/):** A standalone application for extracting and
  decoding Crash Log records.
- **[EFI Tool](efi/):** An EFI tool for reading Crash Log records stored in
  ACPI tables from the EFI shell.

## Getting Started

### Installation

1. **Install the Rust Toolchain**

   Before building the project, install the Rust toolchain using
   [rustup](https://rustup.rs/).

2. **Install the CLI Application**

  Run the following command to install the CLI application:

  ```console
  $ cargo install --path app/
  ```

3. **Uninstall the CLI Application**

  To uninstall the CLI application, use:

  ```console
  $ cargo uninstall -p intel_crashlog_app
  ```

> [!NOTE]
> - For building and using the EFI application, refer to
>   [this document](efi/README.md).
> - To use this reference code in your own application, refer to
>   [this document](lib/README.md).

### Usage

- **Extract** the Crash Log from the platform, it can be from the Windows Event
Log or Linux sysfs:

```console
$ iclg extract sample.crashlog
```

By default the iclg tool will extract all the available sources, but the user
can specify which sources can be extracted:

```console
$ iclg extract -s acpi,pmt:crashlog0
```

- **List** all available Crash Log sources in the platform. Each source
supports different capabilities like `extract`, `trigger`, `enable/disable`,
or `rearm`.

```console
$ iclg list
Source         Description             Capabilities
-------------  ----------------------  ---------------------------------------
acpi           ACPI BERT               extract
pmt:crashlog0  PMT endpoint crashlog0  extract, trigger, enable/disable, rearm
```

- **Trigger** a Crash Log collection on-demand. Like the `extract` command,
you can specify individual sources or trigger all sources by default. This
command is only supported on Linux.

```console
$ iclg trigger
```

- **Rearm** a Crash Log trigger. A source can trigger a Crash Log collection
only once per reset cycle; after a collection has been captured, the source
must be reset and rearmed before it will trigger again. Like the `trigger`
command, you can specify individual sources or rearm all sources by default.
This command is only supported on Linux.

```console
$ iclg rearm
```

- **Enable** or **Disable** the Crash Log collection in the platform.
Individual sources can be specified in the CLI as well. These commands are only
supported on Linux.

```console
$ iclg enable
$ iclg disable
```

- **List** all the collected records, in the extracted sample:

```console
$ iclg info sample.crashlog
  #   Record Type      Product  Size    Skt  Die
----- ---------------- -------- ------- ---- ---------
 0-0  MCA              XYZ/all     832     0
 1-0  CRASHLOG_AGENT   XYZ/all      40     0 io0
```

- **Export** the Crash Log content into JSON:

```console
$ iclg decode sample.crashlog
{
    "crashlog_data": {
        ...
    }
}
```

- **List** available commands using the `--help` option:

```console
$ iclg --help
Extract and decode Intel Crash Log records.

Usage: iclg [OPTIONS] <COMMAND>

Commands:
  enable   Enable Crash Log collection in the platform
  extract  Extract the Crash Log records from the platform
  decode   Decode Crash Log records into JSON
  disable  Disable Crash Log collection in the platform
  info     List the Crash Log records stored in the input file
  list     List the Crash Log sources that are available in the platform
  rearm    Rearm a Crash Log trigger in the platform
  trigger  Trigger an on-demand Crash Log collection in the platform
  unpack   Unpack the Crash Log records stored in the input file
  triage   Triage the Crash Log records stored in the input files
  help     Print this message or the help of the given subcommand(s)

Options:
  -c, --collateral-tree <dir>  Path to the collateral tree. If not specified, the builtin collateral tree will be used
  -v, --verbose...             Sets the verbosity of the logging messages. -v: Warning, -vv: Info, -vvv: Debug, -vvvv: Trace
  -h, --help                   Print help
```

### Development

Instructions for building, testing, and submitting changes are documented in
the following sections:

- [Command Line Interface](app/README.md#Development)
- [EFI Application](efi/README.md#Development)
- [Library](lib/README.md#Development)
