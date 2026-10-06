> [!IMPORTANT]
> **This is an unofficial fork of [EFF's Rayhunter](https://github.com/EFForg/rayhunter) for the Inseego MiFi M2100 5G UW.**
> It is not affiliated with or endorsed by EFF, and the M2100 is not a supported Rayhunter device.
> For EFF's official releases and supported hardware, use the upstream project.

## Rayhunter on the Inseego M2100

This branch (`m2100-support`) runs Rayhunter on the M2100: it captures and analyses live LTE traffic and alerts on detections. The full story of the port, including what we found, what we verified and where we stopped, is in the write-up: **[Rayhunter on the Inseego M2100: field notes from a port](https://tpc.institute/projects/rayhunter-m2100)**.

**What this branch adds to upstream Rayhunter**

| Change | Commit | Applies to |
| --- | --- | --- |
| Diag logging ioctl: the 24-byte msm-4.14 struct | [`5dbd333`](https://github.com/mallorybowes/rayhunter/commit/5dbd333) | likely any 4.14-era Qualcomm device |
| Strip the 8-byte multi-radio header before parsing | [`6a4b0c6`](https://github.com/mallorybowes/rayhunter/commit/6a4b0c6) | modems that send it |
| Keep log versions the parser doesn't know (`0xb114`, `0xb063`) | [`aa04b06`](https://github.com/mallorybowes/rayhunter/commit/aa04b06) | modems sending newer log versions |
| M2100 device support: LED and buzzer alerts, battery | [`5789a1c`](https://github.com/mallorybowes/rayhunter/commit/5789a1c), [`46ae901`](https://github.com/mallorybowes/rayhunter/commit/46ae901) | M2100 |

Bounded parse-error logging, which stops a full log from silently ending a recording, is on its own branch: [`fix-parse-error-log-volume`](https://github.com/mallorybowes/rayhunter/tree/fix-parse-error-log-volume). The device-independent fixes were reported to the Rayhunter maintainers in [discussion #560](https://github.com/EFForg/rayhunter/discussions/560).

**Before you try this**

- **There is no installer.** Getting a root shell on the M2100 currently needs third-party firmware, which also changes settings in the modem itself. We describe what we checked and what we could not, but we do not recommend it to anyone at risk. See [Verifying the firmware](https://tpc.institute/projects/rayhunter-m2100) in the write-up.
- **LTE only, and T-Mobile in practice.** Rayhunter's analysers read LTE; the M2100 has a manual LTE-only mode. AT&T rejects the device.
- **A touchscreen UI exists but isn't published.** It's available on request.

**Contact.** For technical questions that can be public, open an issue here. For anything else, including the touchscreen UI, write to research@tpc.institute.

Code in this fork was written with the help of Claude (Anthropic) and tested on two M2100 units. Licensed GPL-3.0, like upstream.

---

# Rayhunter
![Tests](https://github.com/EFForg/rayhunter/actions/workflows/main.yml/badge.svg)

![Rayhunter Logo - An Orca taking a bite out of a cellular signal bar](https://www.eff.org/files/styles/media_browser_preview/public/banner_library/rayhunter-banner.png)

Rayhunter is a project for detecting IMSI catchers, also known as cell-site simulators or stingrays. It was first designed to run on a cheap mobile hotspot called the Orbic RC400L, but thanks to community efforts, it can [support some other devices as well](https://efforg.github.io/rayhunter/supported-devices.html).
It's also designed to be as easy to install and use as possible, regardless of your level of technical skills, and to minimize false positives. 

&rarr;  Check out the [installation guide](https://efforg.github.io/rayhunter/installation.html) to get started.

&rarr; To learn more about the aim of the project, and about IMSI catchers in general, please check out our [introductory blog post](https://www.eff.org/deeplinks/2025/03/meet-rayhunter-new-open-source-tool-eff-detect-cellular-spying). 

&rarr; For discussion, help, or to join the mattermost channel and get involved with the project and community check out the [many ways listed here](https://efforg.github.io/rayhunter/support-feedback-community.html)!

&rarr; To learn more about the project in general check out the [Rayhunter Book](https://efforg.github.io/rayhunter/).

**LEGAL DISCLAIMER:** Use this program at your own risk. We believe running this program does not currently violate any laws or regulations in the United States. However, we are not responsible for civil or criminal liability resulting from the use of this software. If you are located outside of the US please consult with an attorney in your country to help you assess the legal risks of running this program.

*Good Hunting!*
