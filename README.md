# Buddhabrot
<p align="center">
  <img width=300 height=300 alt="Uh oh, the example picture didn't load" src="https://github.com/WilliamASumner/buddha/raw/master/examples/fractal.png" />
</p>

## Running it
Running Buddha should be as easy as:
```
git clone https://github.com/WilliamASumner/buddha
cargo run --release
```
Running with the `--release` flag is optional but will probably make it run faster. All output is directed to the `output/fractal.png` file, though this will probably change.

## Description
Buddha is my experiment with a "3D" form of the mandelbrot set.
### Regular Mandelbrots
To create a typical mandelbrot image, every pixel in the image is associated with a set of coordinates/an imaginary number. This number is then run through an iterative equation, often cited in the form z<sub>k+1</sub>^2 = z<sub>k</sub>^2 + c, with c being the initial pixel coordinates and z being an evolving value. To test if a point is in the set, all you have to do is to run the numbers through this equation for X iterations, and anything that "escapes" before iteration X (anything with a magnitude > 2) is added to the set. The coloring is often calculated from the number of iterations a point reached before escaping (i.e. points NOT in the set contribute to all the colors in the fractal plots you can find online).

### The Buddhabrot
The idea behind the Buddhabrot is to also take into account the points visited before a certain c escapes, i.e. the previous iterations of z. Coloring then comes from the number of times a particular pixel is visited rather than how long an sequence with c equal to the value of the pixel. This gives the very nebulous and smoky looking appearance of the higher iteration Buddhabrots found in the examples folders. Other coloring schemes run the iterations with various max values for different color channels, but I haven't implemented that yet.

### Optimizations
In the sources I've found, there are a couple of interesting trade offs that can be considered when rendering these images.
#### Metropolis Sampling
One particular optimization is the use of the [Metropolis-Hastings algorithm](https://en.wikipedia.org/wiki/Metropolis%E2%80%93Hastings_algorithm). This algorithm basically allows the user to define how "interesting" a particular orbit is (usually by how close its iteration count is to the max iteration cut off) and reuse it to make similar samples. This technique is really good for generating zoomed in portions of the Buddhabrot, because it can favor samples that affect the zoomed in area instead of wasting time on samples outside of the desired image portion (samples need to come from the whole space still because the orbits can go anywhere). The drawback is that images of the whole set, it causes the sampler to focus on similar orbits which produce (arguably) less interesting results, though they are still clearer than they would've been.

#### Cardioid and Main Bulb
One very nice and simple optimization is to ignore points known to be in the set a priori (i.e. we know they will escape). By ignoring points inside the main cardioid and first bulb we avoid generating many samples that will reach the max iteration amount.

#### Orbit Caching
I'm not aware of any official term for this technique, but I think the name orbit caching fits. During the orbit calculation you can store the points visited so they can be used to calculate the final image without having to reconstruct the path. This works very nicely for small max iteration values, but can quickly result in high memory usage if many points close to the max iteration amount are stored, so a tradeoff is needed.

## Further Reading
A lot of the ideas mentioned here I found from other sources, be sure to read through some of them if my explanations weren't enough.
1. [Melinda Green's Site](http://superliminal.com/fractals/bbrot/bbrot.htm)
2. [softologyblog](https://softologyblog.wordpress.com/2011/06/26/buddhabrot-fractals/)
3. [Alexander Boswell's Site](http://www.steckles.com/buddha/)
4. [Cupe's amazing post on generating a hi-res Buddhabrot](https://erleuchtet.org/2010/07/ridiculously-large-buddhabrot.html)
5. [Numberphile's Video Introduction to the Mandelbrot Set] (https://www.youtube.com/watch?v=NGMRB4O922I)
6. [Numberphile's Video on Mandelbrot Orbits] (https://www.youtube.com/watch?v=FFftmWSzgmk)
