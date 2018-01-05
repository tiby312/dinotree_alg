



[![Build Status](https://travis-ci.org/tiby312/collie.svg?branch=master)](https://travis-ci.org/tiby312/collie)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](https://github.com/tiby312/collie)








#step0   - have a list of bots
	xxxxxxxx
#step1  - create a list of pointers to those bots
	xxxxxxxx
	^^^^^^^^
	pppppppp
#step2 - perform the element swap intensive creation a kdtree using the pointers.

#step3 - create the dynamic tree.




# collie

.

# KdTree Algorithm Overview

An iterative mulithreaded hybrid kdtree/mark and sweep algorithm used for broadphase detection.


# KdTree Construction

The bot vec is binned and sorted so that logically is laid out like this:
   
    //pre order has better space locality as you traverse down the tree
    //   |===|-------------------|----------------------------|
    //       |===|--------|------|=====|-----------|----------|
    //           |==|--|--|=|-|--|     |====|---|--|===|--|---|
    //              |==|==| |=|==|          |===|==|   |==|===|
The bots in each node are also sorted alone alternative axises.





# Colliding pairs querying



# Rectangle Querying


# Multi Rectangle Querying



# Extensions
The limitation of the current design is the level of indirection between the tree,
and the slices of bots in each node. Currently it is a reference to a slice in a seperate vec.

Currently, the bots are allocated once, and then every iteration the tree is allocated. 
This means that depending on where the tree gets allocated on that particular tick, you might get different performance, since the tree will had references to the bots. 

One thing that can be done to alleviate this is make sure that the bot vec is as close as possible to the tree in memory. This isnt done in this implementation. 


Another limitation is the amount of dead memory that is allocated in the leaf nodes.
Many of the fields in the node struct, are uneeded for the child nodes. This wouldn't be so bad 
if it wernt for the fact that this is a complete tree, so there are a LOT of leaf nodes.
Allocation of a seperate tree that has one less height is appealing, but it introduces a level
of indirection. Ideally, there would be a tree data structure that took as type arguments a Node
and a LeafNode type. 


In order to speed up collision finding, the bots could be stored directly in the tree.
This would mean that every node might have a different size. 
The nodes would be dst's like this:
struct Node{
	divider:f32,
	bots[Bot] //Not a pointer to a slice of bots, or a pointer to a slice of bot references.
}
Constructing this tree would have some downsides. It would have to be done sequentially. 
The starting position of every node would depend on constructing all the ones behind it
(in bfs order). It would also require a lot of manual pointer manipulation in order to create
&Nodes.

So while querying would be faster, constructing would be slower. But since construction
takes a less time that query, it may be worth it. 
In this impl, on my laptop, with 100,000 bots. seq rebal and parallel query take about the same time. par rebal on the other hand is about 5 times as fast as par query. Eventually 
the sequential rebal would dominate for very large N.


There is performance improvement if we only allowed bots of the same size to be inserted into the tree. Then we would not need to maintain a range for each node to indicate the range within which all the bots that touch that nodes divider are contained in, because we could just assume
it is the max size. Also the bots themselves wouldnt need to maining a Rect. But of course
you loose generality. Being able to insert bots of arbitrary sizes was a goal of mine.



So the dividers are cached and iterated upon. Coliding pairs could also be cached.
All the bots could be given a loose bounding box that is only updated when then leave it,
or after a set number of iterations.
I chose not to do this. For one thing, I did not want to bound the bots by a maximum speed.
I also didnt want the system's performance to be tied to how fast bots were moving.
Currently the performance of the system is tied to how many bots are colliding.
This is the benefit of the naive method in that it has very consistent performance.


Actual invariants of the tree:
	every node is sorted along an axis that alternates with each level in the tree
	all the bots in a node intersect with its divider.
	all the bots to the left of a node's divider can be found somewhere in its left child, and ditto for right.
	every node keeps track of the range within which all its bots live in along its partitioning axis.

	technical:
	each node of the tree is laid out in memory in bfs order in contiguous memory.
	each node has variable size. it contains the divider, and the containing range, and also all the bots within that node, all in contiguous memory.
	each node's children are pointed to via mutable references.

Pair finding within one node is done via sweep and prune. The bots within a node are sorted
alone the dividing line. This axis is the opposite axis to the one the divider is diving against.
This is done because all the bots in a node are necessarily colliding with the diving line. If they
didn't, they wouldn't live in this node and would live in one of the children nodes. So all the bots
line somewhere on this one line. This made using sweep and prune a good algorithm to choose.

Pair finding between nodes is a different problem from normal pair finding in that we dont need to find the colliding pairs of the bots within one node, only the pairs that collide with bots from the other node. This algorithm is split up into two cases. In the cases where the nodes happen to be sorted on the same axis, it is solved in a mark and sweep manner. In the case where they are along differnt axis, we first get the ranges within both sorted list that actually intersect with the other node's line. 
This eliminates a lot of nodes that dont need to be considered since there is no hope of them colliding with any of the boths in the other node. Unfortunately, finding the pairs within these smaller lists has to be done naively. 


Another reason sweep and prune is well suited here is that the algorithm does not need any kind of data structure besides a stack. It also works on a variable amount of bots. The number of bots in one node could change wildly. At one point I had a oned version of the kdtree inside of each node to do this, but this required dynamically allocating a specialized kdtree with a particular height for each indiviual node. 

The design decision was made that the axis at each level of the tree be known at compile time. There is an XAXIS struct and a YAXIS struct that are passed as type parameters recursively. The benefit of this is that branches that are made based off of the axis can be eliminated at compile time. Specilaized versions of these functions can be generated by the compiler that do not have to branch based off of different axis comparisons. The downside of this is that you have to pick a starting axis statically. This means that if the space that you are partitioning can vary in dimension, you may not be picking the best starting axis to partition against. So if this is a problem for you, you can wrap the collision system behind a trait that does NOT take the starting axis as a type parameter. Then you you can have some dynamic code that will create either a XAXIS, or YAXIS starting collision system that is then returned as a Box of that trait. The downside to this, however, is that two whole versions of your collision system will be monomorphized by the compiler, one for each axis. So this might lead to a big jump in the size of the program. 

A good multi-crate project is setup so that the interface between crates is very simple compared to the complexity contained within each one. 


## License

Licensed under the terms of MIT license and the Apache License (Version 2.0).

See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE) for details.
