[![Build Status](https://travis-ci.org/tiby312/collie.svg?branch=master)](https://travis-ci.org/tiby312/collie)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](https://github.com/tiby312/collie)

An iterative mulithreaded hybrid kdtree/mark and sweep algorithm used for broadphase detection.


# Goal
Create a fast and simple to use broad-phase collision system whose running time did not depend on the size of the 2d space
in which the collision finding functionality was being provided. Does not suffer from "teapot in a stadium" problem.


# Graphs



![chart](./docs/assets/theory.png)

This shows the number of comparisions of each algorithm. The naive pair finding algorithm grows so fast that I stopped computing it early on. The sweep and prune algorithm does much better than the naive. The dinotree version does even better. The sporadic jumps in the dinotree algorithm correspond to the points where the height of the tree increased by one.


![chart](./docs/assets/bench.png)

Unlike the previous graph, this one measure computation time. So this data is very hardware dependent. This data was captured on a dual core dell xps 13 laptop. The same trends as those in the comparison graph are present.






![chart](./docs/assets/tree3d.png)

The above graph shows how


![chart](./docs/assets/sweep3d.png)


## License

Licensed under the terms of MIT license and the Apache License (Version 2.0).

See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE) for details.
