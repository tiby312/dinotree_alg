[![Build Status](https://travis-ci.org/tiby312/collie.svg?branch=master)](https://travis-ci.org/tiby312/collie)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](https://github.com/tiby312/collie)

An iterative mulithreaded hybrid kdtree/mark and sweep algorithm used for broadphase detection.


# Goal
Create a fast and simple to use broad-phase collision system whose running time did not depend on the size of the 2d space
in which the collision finding functionality was being provided. Does not suffer from "teapot in a stadium" problem.


# Analysis

Please see the ![dinotree_report](https://github.com/tiby312/dinotree_report) github project for a writeup of the design and analysis of the algorithms in this project.


## License

Licensed under the terms of MIT license and the Apache License (Version 2.0).

See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE) for details.
