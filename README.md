# gravity-sim-v2
Gravity simulator using Newton's Law of Universal Gravitation, made to teach physics

The goal is for this to run on the web and to otherwise be as easily accessible as possible.

Right click to show the GUI and press escape to hide it. Right clicking on a body shows a body-specific GUI.

# gifs

The start scenario has a simple orbit
![](orbit.gif)

There are previews/predictions when you create a new body

The orbit changes due to integration error, but the timestep is adjustable
![](flawed_orbit.gif)

The GUI uses `imgui-rs`; using it you can create new bodies, adjust existing bodies, or change variables like the timestep.

Negative mass and negative timestep are cool

![](cluster.gif)
