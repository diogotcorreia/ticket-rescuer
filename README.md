# FlySAS pkpass

This is a simple CLI tool to generate a `.pkpass` file for a Scandinavian Airlines flight.

## Usage

Simply run the tool (e.g.,  clone it and run `cargo run`).
It will prompt you for a booking number and last name, and, if successful,
will place the boarding pass in the current directory.

## Why?

SAS only provides PDF tickets through their website, which are very cumbersome to use.
As an alternative, they also provide their mobile app, has the AZTEC ticket but requires
me to install their app, which I find completely unnecessary (as a question of principle).
The only way to obtain an official pkpass from them, is to use their iOS app (which requires
an Apple device, which I don't have) to add it to Apple Wallet and then export it from there.

This CLI tries to generate a very similar boarding pass to the original one, although
with some limitations (see below).

## Limitations

Here are the differences between this and the official pkpass from SAS.
There are plans to improve this over time, and contributions are welcome.

- No SAS logo/icon
- No location coordinates/time (used by iOS to surface the boarding pass at the airport)
- Not signed (for obvious reasons)
- Only reservations with a single passenger and flights are supported (i.e., no round trips, no connecting flights)
