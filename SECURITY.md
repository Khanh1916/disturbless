# Security policy

## Supported versions

Only the latest release gets fixes.

| Version | Supported |
| --- | --- |
| Latest release | Yes |
| Older releases | No |

## What counts as a vulnerability here

Disturbless closes programs, sends a key press to a browser, and writes the Windows hosts file through an elevated process. A flaw in any of those is serious. Examples: a way to make the app close a process it must never touch, a way for page content to make it act on an arbitrary process or path, a hosts file edit that outlives the session or damages other entries, or anything that gives an attacker elevation.

Bypassing the lock on purpose (killing the process from Task Manager, for example) is not a vulnerability. The app is a self-discipline tool, not a security boundary against the person who runs it.

## Reporting

Please do not open a public issue. Use the private report form: open the Security tab of this repository and choose "Report a vulnerability". Include the version, the steps, and what you saw.

You can expect a first reply within a week. Once a fix is released, the report is credited to you unless you prefer to stay anonymous.
