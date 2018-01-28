[![Build Status](https://travis-ci.org/tiby312/collie.svg?branch=master)](https://travis-ci.org/tiby312/collie)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](https://github.com/tiby312/collie)



Here is the outline of the usecase of this crate.
#step0   - have a list of bots
	xxxxxxxx
#step1  - create a list of pointers to those bots
	xxxxxxxx
	^^^^^^^^
	pppppppp
#step2 - perform the element swap intensive creation a kdtree using the pointers.

#step3 - construct the dynamic tree in one contiguous block of memory.
		 copy all the bots belonging to each node into the dyn tree.
		 now the tree is laid out in memory for fast querying.

#step4 - query the tree for colliding pairs



#Testing Strategy
A good test is a test that tests with good certainty that a large portion of code is working properly.
Maintaining tests comes at the cost of anchoring down the design of the production code in addition to having to maintain themselves. As a result, making good abstractions between your crates and modules that have very simple and well defined apis is very important. Then you can have a few simple tests to fully excersise an api and verify large amounts of code.

So lets look at this crate. This crate's sole purpose is to provide a method of providing collision detection. So a good high level test would be to compare the query results from using this crate to the naive method (which is much easier to verify is correct). This one test can be performed on many different inputs of lists of bots to try to expose any corner cases. So this one test when fed with both random and specifically tailed inputs to expose corner cases, can show with a lot of certainty that the crate is satisfying the api. 

The tailed inputs is important. For example, a case where two bounding boxes collide but only at the corner is an extremely unlikely case that may never present themselves in random inputs. To test this case, we have to turn to more point-directed tests with specific constructed set up input bot lists. They can still be verified in the same manner, though.

So even though we know the api is being satisfied, we don't really know if the code is actually going down paths we expect it to as designers of the crate. This is where code coverage can be useful. 

So up until now we have only verified the correctness of this algorithm. We still need to verify that it is worth using. So we have to bench it. The crate api provides a way to get the time taken at each level of the tree. This information is given to the user since finding the right hight of the tree is very usecase specific. 




# KdTree Algorithm Overview

An iterative mulithreaded hybrid kdtree/mark and sweep algorithm used for broadphase detection.


# Goal
Create a fast broad-phase collision system whose running time did not depend on the size of the space
in which the collision finding functionality was being provided. 


# Colliding pairs querying



# Rectangle Querying


# Multi Rectangle Querying



# Extensions


Another limitation is the amount of dead memory that is allocated in the leaf nodes.
Many of the fields in the node struct, are uneeded for the leaf nodes. This wouldn't be so bad 
if it wernt for the fact that this is a complete tree, so there are many leaf nodes. Ideally, there would be a tree data structure that took as type arguments a Node
and a LeafNode type. 



So the dividers are cached and iterated upon. Coliding pairs could also be cached.
All the bots could be given a loose bounding box that is only updated when then leave it,
or after a set number of iterations.
I chose not to do this. For one thing, I did not want to bound the bots by a maximum speed.
I also didnt want the system's performance to be tied to how fast bots were moving.
Currently the performance of the system is tied to how many bots are colliding.
This is the benefit of the naive method in that it has very consistent performance.




Actual invariants of the tree:
	every bot belongs to only one node. 
	every node is sorted along an axis that alternates with each level in the tree
	all the bots in a node intersect with its divider.
	all the bots to the left of a node's divider can be found somewhere in its left child, and ditto for right.
	every node keeps track of the range within which all its bots live in along its partitioning axis.

	technical:
	each node of the tree is laid out in memory in bfs order in contiguous memory.
	each node has variable size. it contains the divider, and the containing range, and also all the bots within that node, all in contiguous memory.
	each node's children are pointed to via mutable references.

Every bot will be copied twice. Once into the dyntree, and once out. I believe the cost of two copiess is worth the benefit from removing a layer of indirection when querying a tree. Note that there is a layer of indirection when rebalancing the tree. The algorithms have different properties. Rebalancing requires a lot of swapping, Query requires a lot of reading and iterating through the tree.

Important property of rebal vs query algorithms. The load of the query algorithm will vary depending on how many of the bots are colliding with each other. On the other hand the rebal algorithm should be more consistent. It may speed up and slow down based on how the bots happen to be arranged going into the algorithm. 




Pair finding within one node is done via sweep and prune. The bots within a node are sorted
alone the dividing line. This axis is the opposite axis to the one the divider is diving against.
This is done because all the bots in a node are necessarily colliding with the diving line. If they
didn't, they wouldn't live in this node and would live in one of the decendants nodes. So all the bots
live somewhere on this one line. This made using sweep and prune a good algorithm to choose.


Pair finding between nodes is a different problem from normal pair finding in that we dont need to find the colliding pairs of the bots within one node, only the pairs that collide with bots from the other node. This algorithm is split up into two cases. In the cases where the nodes happen to be sorted on the same axis, it is solved in a mark and sweep manner. In the case where they are along differnt axis, we first get the ranges within both sorted list that actually intersect with the other node's line. 
This eliminates a lot of nodes that dont need to be considered since there is no hope of them colliding with any of the boths in the other node. Unfortunately, finding the pairs within these smaller lists has to be done naively. 


Another reason sweep and prune is well suited here is that the algorithm does not need any kind of data structure besides a stack. It also works on a variable amount of bots. The number of bots in one node could change wildly. At one point I had a oned version of the kdtree inside of each node to do this, but this required dynamically allocating a specialized kdtree with a particular height for each indiviual node. 

The design decision was made that the axis at each level of the tree be known at compile time. There is an XAXIS struct and a YAXIS struct that are passed as type parameters recursively. The benefit of this is that branches that are made based off of the axis can be eliminated at compile time. Specilaized versions of these functions can be generated by the compiler that do not have to branch based off of different axis comparisons. The downside of this is that you have to pick a starting axis statically. This means that if the space that you are partitioning can vary in dimension, you may not be picking the best starting axis to partition against. So if this is a problem for you, you can wrap the collision system behind a trait that does NOT take the starting axis as a type parameter. Then you you can have some dynamic code that will create either a XAXIS, or YAXIS starting collision system that is then returned as a Box of that trait. The downside to this, however, is that two whole versions of your collision system will be monomorphized by the compiler, one for each axis. So this might lead to a big jump in the size of the program. 

A good multi-crate project is setup so that the interface between crates is very simple compared to the complexity contained within each one. 


What about 3d? Making this multi dimensional would have added to the complexity, so the design desision was made to only target 2d. That's not to say one couldn't still take advantage of this system in a 3d simulation. Every bot could store a height field that you do an extra check against in the collision function. The downside is that imagine if there were many bots stacked on top of each other, but you only wanted to query a small cube. Then doing it this way, your query function would have to consider all those bots that were stacked. If there are only a few different height values, one could maintain a seperte 2d dinotree for each level.




Another possible improvement. Instead of each node in the dyntree storing pointers to its children, store pointer offsets to the children.



## License

Licensed under the terms of MIT license and the Apache License (Version 2.0).

See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE) for details.
