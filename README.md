# SIMple-Gravity
Gravity simulator using Newton's Law of Universal Gravitation, made to help teach physics

[![](https://gitlab.com/mkhan45/gravity-sim-v2/badges/master/pipeline.svg?key_text=build&style=flat-square)](https://gitlab.com/mkhan45/gravity-sim-v2/-/pipelines)

Gravity simulator using Newton's Law of Universal Gravitation, made to teach physics as part of my high school senior research project

Click a body to edit it and graph its properties. 

The GUI uses `egui`; using it you can create new bodies, adjust existing bodies, or change variables like the timestep.


# compiling/running

### precompiled binaries

There are binaries for Windows and Linux in the releases tab on Github, for MacOS you'd have to compile yourself, but feel free to send me the compiled binary to put on Github.

### compile yourself
Install the rust compiler toolchain via [rustup](https://rustup.rs/)

cd to the folder of this repo and run `cargo run --release`.


# gifs

Note: these are from before the rewrite. Try the new version yourself at <https://mkhan45.github.io/SIMple-Gravity/>.

The start scenario has a simple orbit
![](orbit.gif)

You can graph various properties such as speed (more to come)
![](graph.gif)

There are previews/predictions when you create a new body

The orbit changes due to integration error, but the timestep is adjustable
![](flawed_orbit.gif)

You can make trails relative to the selected object.
![](relative_trail.gif)

Negative mass and negative timestep are cool
![](cluster.gif)

You can save and load preset scenarios; binary and nested systems are included.
![](saveload.gif)

# details

Made with [`macroquad`](https://github.com/not-fl3/macroquad) and [`bevy_ecs`](https://github.com/bevyengine/bevy/)


Fully inelastic collisions in which the position of the collided body is decided by the mass weighted average position of the two collided bodies. The new radius is decided by adding the volumes.

Uses Verlet integration, with basic Euler integration energy is not conserved so orbits gradually increase in radius whereas with Verlet integration (and I think implicit Euler), all that happens is the orbits slightly shifting.
