[![Build Status](https://travis-ci.org/tiby312/collie.svg?branch=master)](https://travis-ci.org/tiby312/collie)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](https://github.com/tiby312/collie)


# KdTree Algorithm Overview

An iterative mulithreaded hybrid kdtree/mark and sweep algorithm used for broadphase detection.

# Goal
Create a fast broad-phase collision system whose running time did not depend on the size of the space
in which the collision finding functionality was being provided. Does not suffer from "teapot in a stadium" problem.

# Detailed Design

## The Data Structure
Here are the properties of the constructed tree.
  * Every bot belongs to only one node. 
  * All the bots in a node are sorted along an axis that alternates with each level in the tree
  * All the bots in a node intersect with its divider (except for leafs that dont have a divider).
  * All the bots to the left of a node's divider can be found somewhere in its left child, and ditto for right.
  * Every node keeps track of the range within which all its bots live in along its partitioning axis.

Here are some additional properties that are more about the practical layout:
  * Each node of the tree is laid out in memory in bfs order in contiguous memory.
  *	Each node has variable size. it contains the divider, and the containing range, and also all the bots within that node, all in contiguous memory.
  * Each node's children are pointed to via mutable references.


## The Algorithm
Here is algorithm to create the above described tree.
1. Have a list of bots
2. Create a list of pointers to those bots
3. Perform the element swap intensive construction a kdtree using the list of pointers.
4. Convert the tree of pointers into a tree of raw objects by copying each element.
5. Query the tree for colliding pairs.
6. Optionally query the tree for rectangle colliding sections.
7. Copy each element out of the tree back into the list.

Step 3 (the construction of the tree) is done recursively using this algorithm:
1. Place the divider where the median bot is in the current list.
2. Bin the list of bots into 3 categories, left,right and middle, (middle implying it intersects the divider). There is an alternative divider finding strategy where the divider is found after this step instead of before.
3. Sort the middle list along the axis that is orthoganal to the axis the divider is splititing.
4. Recurse left and recurse right potentially in parallel.

Step 4 (the conversion of the tree to remove a level of indirection) can further be broken down into these steps:
1. Iterate through all the nodes in bfs order and copy the bots that are being pointed to for each node into a new node type.
2. Connect all the parent nodes to their children, by iterating backwards.

Step 5 (the querying of the tree) is done recursively using this algorithm:
1. For the bots beloning to the current node check for collision using sweep and prune.
2. Recursively, find all nodes who's bounding containers intersects the current node. Find colliding pairs between the current node's bots and those node's bots. If the two nodes are the same axis, use sweep and prune. If they are different, first find the subset of each lists that intersect with the other node's bounding rectangle. Then naively check every pair.
3. Recurse left, and right potentially in parallel.

# The relationship between rebalancing and querying

Everything is done in the name of speeding up the querying. This is the part algorithm that could dominate very easily depending on how many bots are intersecting each other. But it's not so much that it dominates, but more so that it could vary wildly. We want to avoid the performance of this collision system to vary with the number of bots colliding as much as possible. So this algorithm trades rebalancing time for query time as much as possible. 


Note that there is a layer of indirection when rebalancing the tree. The algorithms have different properties. Rebalancing requires a lot of swapping, If we rebalanced actual objects instead of pointers there would be many more memory reads and writes. Every bot will be copied only twice. Once into the dyntree, and once out. I believe the cost of two copies is worth the benefit from removing a layer of indirection when querying a tree. In cases where few objects intersect and/or the objects in the tree are very very large, this may not be the case. But in any case, the performance hit is a steady hit that does not vary depending on how many bots are colliding.


# More detailed explanation of the query algorithm

Pair finding within one node is done via sweep and prune. The bots within a node are sorted
alone the dividing line. This axis is the opposite axis to the one the divider is diving against.
This is done because all the bots in a node are necessarily colliding with the diving line. If they
didn't, they wouldn't live in this node and would live in one of the decendants nodes. So all the bots
live somewhere on this one line. This made using sweep and prune a good algorithm to choose.


Pair finding between nodes is a different problem from normal pair finding in that we dont need to find the colliding pairs of the bots within one node, only the pairs that collide with bots from the other node. This algorithm is split up into two cases. In the cases where the nodes happen to be sorted on the same axis, it is solved in a mark and sweep manner. In the case where they are along differnt axis, we first get the ranges within both sorted list that actually intersect with the other node's line. 
This eliminates a lot of nodes that dont need to be considered since there is no hope of them colliding with any of the boths in the other node. Unfortunately, finding the pairs within these smaller lists has to be done naively. 


Another reason sweep and prune is well suited here is that the algorithm does not need any kind of data structure besides a stack. It also works on a variable amount of bots. The number of bots in one node could change wildly. At one point I had a oned version of the kdtree inside of each node to do this, but this required dynamically allocating a specialized kdtree with a particular height for each indiviual node. 

The design decision was made that the axis at each level of the tree be known at compile time. There is an XAXIS struct and a YAXIS struct that are passed as type parameters recursively. The benefit of this is that branches that are made based off of the axis can be eliminated at compile time. Specilaized versions of these functions can be generated by the compiler that do not have to branch based off of different axis comparisons. The downside of this is that you have to pick a starting axis statically. This means that if the space that you are partitioning can vary in dimension, you may not be picking the best starting axis to partition against. So if this is a problem for you, you can wrap the collision system behind a trait that does NOT take the starting axis as a type parameter. Then you you can have some dynamic code that will create either a XAXIS, or YAXIS starting collision system that is then returned as a Box of that trait. The downside to this, however, is that two whole versions of your collision system will be monomorphized by the compiler, one for each axis. So this might lead to a big jump in the size of the program. 

# Exploiting temporal locality

There are two divider-finding strategies that can be used. The first one simply uses the median value as the divider. This is useful when there is no previous state, or when there is no relationship in the position of the bots between queries.

Another strategy to exploit temporal locality is by inserter looser bounding boxes into the tree and caching the results of a query for longer than one step. There are really two variants of this. The bounding box could dynamically grow the faster the bot goes. I didnt like this because now the performance of your system depends on the speed of the bots. If just a few bots are going very very fast, it could destroy the performnace of your system. The other option is have each bot have a constant bounding box size. To do this, you now have to bound the velocity of your bots. That's a constrait I didn't want users to have to buy into. Probably the best is a combination of the two. At the end of the day I'm not convinced that cacheing+looser bounding boxes is better than no caching+tight bounding boxes. The other downside is that the cached results cannot be iterated through concurrently. And building up the cached list of bots is also hard to do efficiently when multithreaded.

# Space and Time Complexity

The theoretical time compleity of this algorithm I bet is very hard to calculate and my guess is that it would depend so wildly on the distribution of the position and sizes of the bots that are fed into it. That plus the fact that the algorithm reused past calculations for the dividers, makes it very hard.

That said bounding it by the worst case is easy, because in the worst case every single bot is colliding with every other bot.

Simliarily bounding it by the best case should also be easy. Best case, all the bots live in only leaf nodes, and none of the bots intersect. Interestingly by the pigeon principle, if you have more bots than there are leaf nodes then this best case scenario isnt possible. 

So really, the best and worst case scenario really tell you nothing useful. 


The space complexity, on the other hand, is much easier to figure out.


# Testing Strategy

## Testing correctness

Simply using rust has a big impact on testing. Because of its heavy use of static typing, many bugs are caught at compile time. This translates to less testing as there are fewer possible paths that the produced program can take. Ideally you want your program to be as static as possible and still satisfy whatever function it is supposed to serve.

A good test is a test that tests with good certainty that a large portion of code is working properly.
Maintaining tests comes at the cost of anchoring down the design of the production code in addition to having to be maintained themselves. As a result, making good abstractions between your crates and modules that have very simple and well defined apis is very important. Then you can have a few simple tests to fully excersise an api and verify large amounts of code.

This crate's sole purpose is to provide a method of providing collision detection. So a good high level test would be to compare the query results from using this crate to the naive method (which is much easier to verify is correct). This one test can be performed on many different inputs of lists of bots to try to expose any corner cases. So this one test when fed with both random and specifically tailed inputs to expose corner cases, can show with a lot of certainty that the crate is satisfying the api. 

The tailored inputs is important. For example, a case where two bounding boxes collide but only at the corner is an extremely unlikely case that may never present themselves in random inputs. To test this case, we have to turn to more point-directed tests with specific constructed set up input bot lists. They can still be verified in the same manner, though.

At this point one could say that we were done. We have a pretty good case that the algorithms that this crate provide are correct and satisfy the api. But there is another expectation/design goal that is hard to show. The point of this crate is that it is a very fast collison detection system. So there is the problem of define how fast is fast enough? If I were simply to show that it was faster than the naive method would that be enough? Or do I have to prove it against some other metric?

Also, talk about how we can just test it for isize, and not float since works over anything that implements NumTrait.


## Benching

So even though we know the api is being satisfied, we don't really know if the code is actually going down paths we expect it to as designers of the crate. This is where code coverage can be useful. Where code coerage fails, though, is the fact that even if all control paths are hit, not all possible values of the variables that effect the outcome are hit. It is also useful to come up with a "upholding invariant" function that can be called at any time on the tree after it has been constructed to be sure that it has all the properties that it needs to perform querying on. 

So up until now we have only verified the correctness of this algorithm. We still need to verify that it is worth using. So we have to bench it. The crate api provides a way to get the time taken at each level of the tree. This information is given to the user since finding the right hight of the tree is very usecase specific. It also serves the purpose of proving that the crate is behaving like a tree and is properly dividing and conquering the problem. So by comparing the performance against the naive approach, and possibly other crates, we can prove that is is worth using.

Simply proving that it is better than the naive approach isnt very impressive. We want to prove that the design constructs and complexity used in the crate are actually accomplishing something. Otherwise you're just maintaining complexity for the sake of complexity. In order to show this, the user has the option of turning off an on certain features of the system using generic parameters. The user can turn off and on multithreading and bench them seperately. The user can specify the median finding strategy and bench them seperately. 

Not all features can be turned off however. How can I show that, for example, using sweep and prune within each node, actually sped things up? This is hard to show. I have a pretty good feeling because I can comment out the the code and notice a speed difference, but then the user has to take my word for it. Should all these features we able to turn off via generic parameters? If you did this, you'd eventually end up with the one high level function with a whole mess of generic parameters. At this point I decided that the user can modify the code and test things out on his own if he wants. It would be nice if there could be automated benching to show the performance improvements (would be interesting to see how they vary on different target platforms) brought upon by every single idea in this crate, but the added complexity, doesnt seem worth it. 


# Extensions and Improvements

A current limitation is the amount of dead memory that is allocated in the leaf nodes.
Many of the fields in the node struct, are uneeded for the leaf nodes. This wouldn't be so bad 
if it wernt for the fact that this is a complete tree, so there are many leaf nodes. Ideally, there would be a tree data structure that took as type arguments a Node
and a LeafNode type. This would then require more branching, though. So it could be that the current design is faster anyway.

So the dividers are cached and iterated upon. Coliding pairs could also be cached.
All the bots could be given a loose bounding box that is only updated when then leave it,
or after a set number of iterations.
I chose not to do this. For one thing, I did not want to bound the bots by a maximum speed.
I also didnt want the system's performance to be tied to how fast bots were moving.
Currently the performance of the system is tied to how many bots are colliding.
This is the one benefit of the naive method in that it has very consistent performance.

What about 3d? Making this multi dimensional would have added to the complexity, so the design desision was made to only target 2d. That's not to say one couldn't still take advantage of this system in a 3d simulation. Every bot could store a height field that you do an extra check against in the collision function. The downside is that imagine if there were many bots stacked on top of each other, but you only wanted to query a small cube. Then doing it this way, your query function would have to consider all those bots that were stacked. If there are only a few different height values, one could maintain a seperte 2d dinotree for each level.

Pipelining. It might be possible to pipeline the process so that rebalancing and querying happen at the same time with the only downside being that bots react to their collisions one step later.

Another possible improvement. Instead of each node in the dyntree storing pointers to its children, store pointer offsets to the children.

Smaller memory footprint if stored position+radius instead of AABB, but less versatile.

# Use of Rust

NumTrait. Use of generics.

## Use of Unsafe

moving objects that dont implement copy.
The multirect example. 
split_at_mut()

#talk about parallilizing node bounding box

#talk about parallilizing binning

# talk about determinism. floating point additive

# think about it like a sponge. the lower you go into the tree, the more stable the calculates get 


# General thoughts on optimizing

Always measure code before investing time in optimizing. As you design your program. You form in your mind ideas of what you think the bottle necks in your code are. When you actually measure your program, your huntches can be wildly off.
Dynamic allocation is fast. Dynamically allocate large vecs in one allocation is fast. Its only when you're dynamically allocte thousands of small objects does it become bad.


# Android

talk about ByteBuffer
talk about android NDK.
talk about thread safety
talk about targeting android that uses ART vs Dalvik
talk about avoiding copying and ByteBuffer

# Author notes
As I delved further and further into this passion project. I came to realize that I may have "bitton off more than I could chew" given my lifestyle (9-5 job + girlfriend). So this isn't really as "rigorous" as I would like it to be.


talk about recursion limit

talk about parallel pattern defeating quick select would be nice


talk about guarentees on the number of bots in chidlren ondes when always using the median. I tried many divider placement strategies where divider positions from previous positions were used to push them in the right direction. But at the end of the day, all these strategies of applying "forces" to the dividers suffer from the divider not being exactly where they need to be. They are always moving towards way they should be. I dont think the performance gains of a faster rebalance using this strategy is worth the performance loss from querying using suboptimal divider placement.


## License

Licensed under the terms of MIT license and the Apache License (Version 2.0).

See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE) for details.
