# Yet Another Media Controller
#### (Probably—I never checked for others.)

***

#### TUI for displaying information about and controlling currently playing media.

![Image showing a preview of the application's interface](preview.png)

## Usage


Use <kbd>←</kbd> and <kbd>→</kbd> to skip to the previous and next track, respectively, and <kbd>Space</kbd> to play/pause. The onscreen buttons can also be clicked. 

<!-- Config file has not been set back up yet, leaving for later reference.
## Configuration
The configuration file is located in `~/.config/yamc/yamc.toml`. The below table shows the available options, but know that this will lag behind actual development a bit.

| Key								| Default value		|
|-----------------------------------|-------------------|
| `image_display_backend`			| `'viu'`			|
| `image_acquisition_method`		| `'mpris_arturl'`	|
| `controls_button_prev`			| `'⏮'`				|
| `controls_button_pauseplay`		| `'⏯'`				|
| `controls_button_next`			| `'⏭'`				|
| `controls_bg_active`				| `true`			|
| `controls_bg_cap_left`			| `''`				|
| `controls_bg_cap_right`			| `''`				|
| `controls_bg_length`				| `13`				|
| `controls_bg_cap_padding_left`	| `1`				|
| `controls_bg_cap_padding_right`	| `1`				|
| `image_margin_left`				| `2`				|
| `image_margin_right`				| `3`				|
| `image_margin_top`				| `2`				|
| `image_margin_bottom`				| `3`				|
| `image_size`						| `24`				|
-->


## Obtaining

#### Runtime Dependencies
<!--* [ffmpeg](https://ffmpeg.org) is required for `ffmpeg` cover art extraction, although any player which properly populates the `mpris:artUrl` field with a local file will work properly with the default `mpris_arturl` option.-->
* [Chafa](https://github.com/hpjansson/chafa/) is required to display cover art.

Binaries are not currently provided, so you'll have to build the project yourself.
```sh
git clone https://github.com/lilithium-hydride/yamc
cd yamc
cargo +nightly run
```