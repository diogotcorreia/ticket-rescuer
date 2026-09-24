# Ticket Rescuer

_This repository was previously called "FlySAS pkpass", but has since gotten support for providers other than FlySAS._

Ticket Rescuer is a CLI tool that can generate tickets and boarding passes
in an easily consumable format (e.g., `.pkpass`)
for companies that refuse to provide these files.

## Usage

Simply run the tool (e.g.,  clone it and run `cargo run`).

You will be prompted for which provider to want to get a ticket from,
and then for further information that can be needed (e.g., booking number, last name, email, etc.).

The boarding pass(es) will be saved to the current directory.

## Supported Providers

| Name | File Format | Pass type |
| ---- | ----------- | --------- |
| FlySAS | `.pkpass` | Recreated from API data |
| Ryanair | `.pkpass` | Official file from API |

## Why?

Some airlines (e.g., SAS) only provide PDF tickets through their website, which are very cumbersome to use.
While SAS provides an alternative via their mobile app, which has the AZTEC ticket,
I find that completely unreasonable (as a question of principle).

Other airlines are even worse (e.g., Ryanair), and **force** you to download their app to
get a boarding pass at all.

This CLI tries to generate very similar tickets/boarding passes to the original ones,
or use the official ones when possible,
although with some limitations (see below).

## Contribute

Support for more providers is always welcome.
Please open an issue to coordinate, or reach out via email or other socials (see https://diogotc.com).

## Limitations

Here are the differences between this and the official pkpass from SAS.
There are plans to improve this over time, and contributions are welcome.

- No SAS logo/icon
- No location coordinates/time (used by iOS to surface the boarding pass at the airport)
- Not signed (for obvious reasons)
- Only reservations with a single passenger and flights are supported (i.e., no round trips, no connecting flights)
