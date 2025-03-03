<div align="center">
  <img src="https://github.com/hlsxx/tuicam/blob/master/blob/example2.png" alt="Tuicam colorful" style="width:100%; max-height:400px" />
  <a href="https://git.io/typing-svg"><img src="https://readme-typing-svg.demolab.com?font=Itim&size=25&pause=1000&color=E68F6A&width=435&lines=Tuicam+-+Camera+in+the+terminal" alt="Tuicam" /></a>
</div>

## Requirements

### OpenCV
Before using Tuicam, install [OpenCV](https://opencv.org/) on your computer.  

**Note:** If your OpenCV version is **< 4.6.0**, use the feature flag `opencv_old`:  
```sh
cargo install tuicam --features opencv_old
```

## Installation

### Cargo
Tuicam is available on [crates.io](https://crates.io/crates/tuicam).

```
cargo install tuicam
```

### Nix
```
nix-shell -p tuicam
```

