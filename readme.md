[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](https://github.com/tiby312/collie)

Provides various query aabb broad phase algorithms such as collision pair finding, raycast, or k_nearest, on the [dinotree](https://crates.io/crates/dinotree) data structure. 

### Inner projects

The dinotree_alg_demo inner project is meant to show case the use of these algorithms. It depends on the piston 2d engine to draw to the screen. 

The dinotree_alg_data project generates some graphs using RustGnuPlot. These graphs are used to create the reports in the dinotree_report project that is a seperate dinotree project.

### Analysis

Please see the [dinotree_report](https://github.com/tiby312/dinotree_report) github project for a writeup of the design and analysis of the algorithms in this project.

### Rust Version

Requires rust nightly for the following features:
~~~~text
#![feature(test)]
#![feature(trusted_len)]
~~~~

### License

Licensed under the terms of MIT license and the Apache License (Version 2.0).

See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE) for details.


### Pictures

These are pictures from the inner dinotree_alg_demo project.    

#### k_nearest    
![chart](https://github.com/tiby312/dinotree_report/blob/master/pictures/pic1.png)
#### multirect
![chart](https://github.com/tiby312/dinotree_report/blob/master/pictures/pic2.png)
#### raycast
![chart](https://github.com/tiby312/dinotree_report/blob/master/pictures/pic3.png)
#### raycast f64
![chart](https://github.com/tiby312/dinotree_report/blob/master/pictures/pic4.png)
#### nbody
![chart](https://github.com/tiby312/dinotree_report/blob/master/pictures/pic5.png)
#### colfind
![chart](https://github.com/tiby312/dinotree_report/blob/master/pictures/pic6.png)
#### intersect_with
![chart](https://github.com/tiby312/dinotree_report/blob/master/pictures/pic7.png)