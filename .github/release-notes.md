**A test build, for loading by hand.** The files are not signed. Pocket Instruments is not in the VCV Library and has not been submitted to it. One instrument so far: Rhythm, a sixteen-step drum machine.

**No install:** play it in a browser at <https://kinggs.github.io/pocket-instruments/web/rhythm.html> (made for a phone; a desktop browser works too).

## Install it in VCV Rack 2

1. Download the `.vcvplugin` file for your computer:
   - Windows: `…-win-x64.vcvplugin`
   - Linux: `…-lin-x64.vcvplugin`
   - Mac with Apple silicon (M1 and later): `…-mac-arm64.vcvplugin`
   - Mac with an Intel processor: `…-mac-x64.vcvplugin`
2. In Rack, choose `Help > Open user folder`. Put the file in the folder there whose name ends in your platform: `plugins-win-x64`, `plugins-lin-x64`, `plugins-mac-arm64` or `plugins-mac-x64`. Leave it as it is: Rack unpacks it itself.
3. Restart Rack. It unpacks and loads the plugin when it starts.
4. Download `Showcase.vcv`, open it in Rack (`File > Open`) and press play on the Rhythm module. Besides Rhythm it uses only VCV's own Core and Fundamental modules.

On a Mac, macOS may refuse to load an unsigned download. This is not yet tested: see [issue 83](https://github.com/kinggs/pocket-instruments/issues/83).

## Check a file

Each file has a build attestation tying it to this repository and the run that built it. With the GitHub CLI:

```
gh attestation verify <file> -R kinggs/pocket-instruments
```

## Licence and credits

MIT. The plugin's effects port code from Emilie Gillet's stmlib and Plaits (MIT); her notice is in the plugin, at `res/stmlib-plaits-MIT.txt`. The typeface, JetBrains Mono, comes with its licence (OFL) at `res/fonts/OFL.txt`. The sixteen starter samples are CC0. All credits: [CREDITS.md](https://github.com/kinggs/pocket-instruments/blob/main/CREDITS.md).

An affectionate, non-commercial homage to Teenage Engineering's pocket operators. Not affiliated with Teenage Engineering. Their names are used only to describe.
