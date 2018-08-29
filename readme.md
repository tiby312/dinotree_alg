[![Build Status](https://travis-ci.org/tiby312/collie.svg?branch=master)](https://travis-ci.org/tiby312/collie)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](https://github.com/tiby312/collie)

An iterative mulithreaded hybrid kdtree/mark and sweep algorithm used for broadphase detection.


# Goal
Create a fast and simple to use broad-phase collision system whose running time did not depend on the size of the 2d space
in which the collision finding functionality was being provided. Does not suffer from "teapot in a stadium" problem.


# Graphs



![chart](../assets/bench.png)

So this data is very hardware dependent. This data was captured on a dual core dell xps 13 laptop. The naive pair finding algorithm grows so quickly, I stopped computing it at a certain point. The sweep and prune algorithm grows much slower than the naive algorithm, but the dinotree pair finding algorithm is still faster. The parallel version is also slighty faster.


![chart](../assets/theory.png)

This shows the number of comparisions of each algorithm. This is not machine dependant. The same trends from the benching are noticable. The sporadic jumps in the dinotree algorithm correspond to the points where the height of the tree increased by one.


![chart](../assets/sweep3d.png)


![chart](../assets/tree3d.png)

## License

Licensed under the terms of MIT license and the Apache License (Version 2.0).

See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE) for details.
