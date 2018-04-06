[![Build Status](https://travis-ci.org/tiby312/collie.svg?branch=master)](https://travis-ci.org/tiby312/collie)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](https://github.com/tiby312/collie)

An iterative mulithreaded hybrid kdtree/mark and sweep algorithm used for broadphase detection.





# Goal
Create a fast and simple to use broad-phase collision system whose running time did not depend on the size of the 2d space
in which the collision finding functionality was being provided. Does not suffer from "teapot in a stadium" problem.



# The Data Structure itself
First lets talk about the internal data structure that is used to actually accomplish the collision detection.
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

So the entire tree and the elements belong to each node and all in one piece of contiguous memory.

# Creation of the Data Structure

Creating the tree is made up of these steps:
1. Find the median in the list of bots.
2. Bin the bots into three bins. those to the left of the median, those that touch the median, 
and those to the right of the median.
3. Sort the middle list along the axis that is orthoganal to the axis the divider is splititing.
4. Recurse left and right in parallel

# Finding all colliding pairs

1. For the bots beloning to the current node check for collision using sweep and prune.
2. Recursively, find all nodes who's bounding containers intersects the current node. Find colliding pairs between the current node's bots and those node's bots. If the two nodes are the same axis, use sweep and prune. If they are different, first find the subset of each lists that intersect with the other node's bounding rectangle. Then naively check every pair.
3. Recurse left, and right potentially in parallel.

So the querying sort of does "two pases" of the tree. For each node, it will collide with all the bots in that node, then it will look for all children nodes that intsect with itself and collide with those bots. Then that node is completely done. So it can be removed, and now you have two completely independant trees that you can repeat the algorithm on.

# Finding bots in a rectangle

todo

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


# Optimizations: The relationship between rebalancing and querying

Everything is done in the name of speeding up the querying. This is the part algorithm that could dominate very easily depending on how many bots are intersecting each other. Well, it's not so much that it dominates, but more so that it could vary wildly. We want to avoid the performance of this collision system to vary with the number of bots colliding as much as possible. So this algorithm trades rebalancing time for query time as much as possible. 

# Exploiting temporal locality

There's really two different contexts in which temporal locality can be talked about. There's the time locality between states of the 2d world between calls to create and destroy the tree. Then there's the time locality of the internal locality as it makes it ways through the alogrithm.

The sort answer? It does not. For a while I had the design where the dividers would move as those they had mass. They would gently be pushed to which ever side had more bots. The problem with this approach is that the divider locations will mostly of the time be sub optimial. And the cost saved in rebalancing just isnt enough for the cost added to querying with a suboptimal partitioning. By always partitioning optimally, we can make some more assumptions. For example, we are guarenteed that the leaf nodes will have less than 10 bots (because we picked the height of the tree specifically so that that would be the case). Therefore, when sorting the leaf nodes, we can do a simple insertion sort, instead of doing a more general sorting algorithm that might branch into doing an insertion sort.

Another strategy to exploit temporal locality is by inserting looser bounding boxes into the tree and caching the results of a query for longer than one step. There are really two variants of this. The bounding box could dynamically grow the faster the bot goes. I didnt like this because now the performance of your system depends on the speed of the bots. If just a few bots are going very very fast, it could destroy the performnace of your system, and this went against my design goal of creating a very consisten performing system. The other option is have each bot have a constant bounding box size. To do this, you now have to bound the velocity of your bots. That's a constrait I didn't want users to have to buy into. Probably the best is a combination of the two. At the end of the day I'm not convinced that cacheing+looser bounding boxes is better than no caching+tight bounding boxes. The other downside is that the cached results cannot be iterated through concurrently. And building up the cached list of bots is also hard to do efficiently when multithreaded.


So in short, this system doesnt take advantage of temporal locality, but the user can still take advantage of it by inserting loose bounding boxes and then querying less frequently to amortize the cost. I didnt explore this since I need to construct the tree every iteration anyway in my android demo, because I wanted the feedback of the user moving his finger around to be imeddiate. So to find all the bots touching the finger i need the tree to be up to date every single iteration. This is because I have no way of know where the user is going to put his finger down. I cant bound it by velocity or acceleration or anything. If I were to bound the touches "velocity", it would feel more slugish i think. It would also delay the user putting a new touch down for one iteration possibly.



# Testing correctness

Simply using rust has a big impact on testing. Because of its heavy use of static typing, many bugs are caught at compile time. This translates to less testing as there are fewer possible paths that the produced program can take. Also the fact that the api is generic over the underlying number type used is useful. This means that we can test the system using integers and we can expect it to work for floats. It is easier to test with integers since we can more easily construct specific scenarios where one number is one value more or less than another.

A good test is a test that tests with good certainty that a large portion of code is working properly.
Maintaining tests comes at the cost of anchoring down the design of the production code in addition to having to be maintained themselves. As a result, making good abstractions between your crates and modules that have very simple and well defined apis is very important. Then you can have a few simple tests to fully excersise an api and verify large amounts of code.

This crate's sole purpose is to provide a method of providing collision detection. So a good high level test would be to compare the query results from using this crate to the naive method (which is much easier to verify is correct). This one test can be performed on many different inputs of lists of bots to try to expose any corner cases. So this one test when fed with both random and specifically tailed inputs to expose corner cases, can show with a lot of certainty that the crate is satisfying the api. 

The tailored inputs is important. For example, a case where two bounding boxes collide but only at the corner is an extremely unlikely case that may never present themselves in random inputs. To test this case, we have to turn to more point-directed tests with specific constructed set up input bot lists. They can still be verified in the same manner, though.

# Benching

Writing benches that validate every single piece of the algorithm design is a hassle ,although it would be nice. Ideally you dont want to rely on my word to say that, for example, using sweep and prune to find colliding pairs actually sped things up. It could be that while the algorithm is correct and fast that this particular aspect of the algorithm actually slows things down. 

So I dont think writing tons of low level benches are worth it. If you are unsure of a piece of code, you can bench the algorithm as a whole, change a piece of the algorithm, and bench again and compare results. Because at the end of the day, we already tested the correctness, and that is the most important thing. So I think this strategy, coupled with code coverage and just general reasoning of the code can supplement tons of benches to validate the innards of the algorithm.


# Extensions and Improvements

A current limitation is the amount of unused allocated memory that is allocated in the leaf nodes.
Many of the fields in the node struct, are uneeded for the leaf nodes. This wouldn't be so bad 
if it wernt for the fact that this is a complete tree, so there are many leaf nodes. Ideally, there would be a tree data structure that took as type arguments a Node
and a LeafNode type. This would then require more branching, though. So it could be that the current design is faster anyway.

What about 3d? Making this multi dimensional would have added to the complexity, so the design desision was made to only target 2d. That's not to say one couldn't still take advantage of this system in a 3d simulation. Every bot could store a height field that you do an extra check against in the collision function. The downside is that imagine if there were many bots stacked on top of each other, but you only wanted to query a small cube. Then doing it this way, your query function would have to consider all those bots that were stacked. If there are only a few different height values, one could maintain a seperte 2d dinotree for each level. Looking at the real world though, and most usecases, your potential z values are much less than our potetial x and y values. So for many cases, it probably better to use the tree for 2 dimentions, and then naively handling the 3rd. Then you dont suffer from the "curse of dimensionality"?

Pipelining. It might be possible to pipeline the process so that rebalancing and querying happen at the same time with the only downside being that bots react to their collisions one step later.

Another possible "improvement" would be to store positions and radius instead of bounding boxes.
That saves one extra float, but it is less versatile. Also fixing the radius of the bots would be a
huge performance improvement. Every bot would only need to store a position then.


# Use of Unsafe

moving objects that dont implement copy.
The multirect example. 
split_at_mut()

# think about it like a sponge. the lower you go into the tree, the more stable the calculates get 


# Talk about bfs vs dfs ordering. Better space locality if in dfs ordering.


# General thoughts on optimizing


Optimization is a great balancing act. There are so many interesting questions. Indirection, locality, dividng and conquering. Every algorithm has unique properties. On top of the designing the algorithm, there is a whole nother level of interesting questions when it comes to implementing said algorithm. How to best write the code for maintainability, readabilty, performance, simplicity in the api. Concurrency is the strangest of all. The theory and in practice clash together. I think coding is well suited for people who like chess. The same interesting desision making takes place. "big picture" thinking is very important, and you are often rewarded for following your intuition and hunches down some path. Making desisions of when to stabailize and write tests, or when to capitalize on some new design oportunity, it goes on.




Always measure code before investing time in optimizing. As you design your program. You form in your mind ideas of what you think the bottle necks in your code are. When you actually measure your program, your huntches can be wildly off.

Dynamic allocation is fast. Dynamically allocating large vecs in one allocation is fast. Its only when you're dynamically allocting thousands of small objects does it become bad. Even then, probably the allocations are fast, but because the memory will likely be fragmented, iterating over a list of those objects could be very slow. Concepually, I have to remind myself that if you dynamically allocate a giant block, its simply reserving that area in memory. There isnt any expensive zeroing out of all that memory unless you want to do it. That's not to say the complicated algorithms the allocator has to do arn't complicated, but still relatively cheap.

The thing is that if you don't use dynamic allocation, and you instead reserve some giant piece of memory for use of your system, then that memory is not taken advanage of when your system is not using it. It is wasted space. If you know your system will always be using it then sure it is fine. But I can see this system being used sometimes only 30 times a second. That is a lot of inbetween time where that memory that cannot be used by anything else. So really, the idea of dynamic allocation only works is everybody buys into the system. Another option is to make your api flexible enough that you pass is a slice of "workspace" memory, so that the user can decide whether to dynamically allocate it everytime or whatever. But this complicates the api for a very small portion of users who would want to not using the standard allocator.

When dealing with parallelism, benching small units can give you a warped sense of performance. Onces the units are combined, there may be more contention for work stealing. With small units, you have a false sense that the cpu's are not busy doing other things. For example, I parallalized creating the container range for each node. Benchmarks showed that it was faster. But when I benched the rebalancing as a whole, it was slower with the parallel container creation. So in this way, benching small units isnt quite as useful as testing small units is. That said, if you know that your code doesnt depend on some global resource like a threadpool, then benching small units is great.

Platform dependance. Rust is a great language that strives for platform independant code. But at the end of the day, even though rust programs will behave the same on multiple platforms, their performance might be wildly different. And as soon as you start optimizing for one platform you have to wonder whether or not you are actually de-optimizing for another platform. For example, rebalancing is much slower on my android phone than querying. On my dell xps laptop, querying is the bottle neck instead. I have wondered why there is this disconnect. I think part of it is that rebalancing requires a lot of sorting, and sorting is something where it is hard to predict branches. So my laptop probably has a superior branch predictor. Another possible reason is memory writing. Rebalancing involves a lot of swapping, whereas querying does involve in any major writing to memory outside of what the user decides to do for each colliding pair. In any case, my end goal in creating this algorithm was to make the querying as fast as possible so as to get the most consistent performance regardless of how many bots were colliding.

In fact, I ended up with 3 competing rebalancing algorithms. The first one would simply create pointers to all the bots, and sorted the pointers. This one was the slowest. I think it is because only one field is relevant to this algorithm, the bounding box rect. So all the other fields were just creating space that needed to be jumped over. So the distance between relevant information to be used by the algotihm wa high. On the other hand this method didnt have to allocate much memory. There is also the problem that its highly dependant on where the given slice is in memory. If its far away from the vec of pointers, then every deref is expensive, probably.

The second one would create a list of rects and ids pulled from the bots and sort that. The main characteristic of this method is that there is no layer of indirection. The downside is that swapping elements is more expensive since you are not swapping pointers, you are swapping bounding boxes coupled with an id. So this method made the median-finding part of the algorithm very fast, but made the sorting slower.

The third method was to create a list of rects, and then create a list of pointers to that list of rects and then sort that. The obvious downside is that you end up dynamically allocating two seperate vecs. But really it doesnt use any more memory that method 2. It has the benefit of swapping only pointers, and it also has better memory locality that method1.

For large numbers of bots 50,000+, the second method seems to be the best on both my phone and laptop.



Parallalization is done using rayon. The rust slice provided split_at_mut() and rayon's join() are two extremely powerful constructs. Seeing and understanding the api and implementation of split_at_mut() is what convinced me that rust was the future.
At a certain point while going down the tree, we switch to a sequential version as recommended by rayon's usage guidelines when used in recursive divide and conquer problems. The depth at which point we switch to sequential should not be static. It should change along with the size of the problem. TODO explain this more.



There are lot of optimization questions that I had to struggle with. For example, I'm not entirely convinced that storing the indexes of the bots in their sorted tree order in a seperate Vec was a good idea. It might be better to store the indicies in the tree themselves along side the actual bots. The downside is that now the dinotree is full of these indicides that are not used for any of the querying and are only needed when restoing the order of the bots when the tree is destroyed. So every query you do on the tree will slower because you lose space locality for no benefit. On the upside, when you restore the tree, you dont have to iterate through 3 seperate data stures in order to restore it. You can iterate through the tree itsef, the index data structure, and the original bot slice in which to move all the bots back into.

The rebalancing algorithm has the problem of working with two seperate segments of memory. On one hand it is swapping bots around in the provided bots slice. And on the other hand it is building of a tree of nodes that point to slices of the before mentioned bots. I think ideally the nodes and the bots would all be in contiguous memory just like the dinotree itself onces rebalancing is finished. I thought about how you might be able to do this, but I don't think it is posible without requiring massive amounts of shifting in memory, so it is porobably not worth it. 



# Android

To create the android gemo, I built native libraries and loaded they dynamically inside a regular android java app. So I didnt use a native activity. I wanted to allow the possibiilty of using android ui in the future. I followed https://github.com/kennytm/rust-ios-android. I think the android-rs-glue crate + glutin crate have a lot of stability issues. (For example see https://github.com/tomaka/android-rs-glue/issues/172). 

So I created a jni interface to my rust demo. In order to make this jni interface platform independant required some unique code in the jni wrapper. If it was a 64bit system, I could simply cast pointers to jlongs and give them to the java side as a handle to the game instance. On 32bit systems there needs to an extra step where the pointer is embeded and extracted from a jlong.

Passing data to and from the jni interface is slow, unless you use a ByteBuffer. Here endianess matters. The Jav bytebuffer has a function to change the byte order to the native byte order. But you first have to populate the bytebuffer with the correct types. Afterall, byte order doesnt make sense without the context of what type it is you are changing the byte order of whether it is a int or a long, etc. So first you have to put() a bunch floats, then change the byte order to native, and then you can pass this byte buffer to the native library for it to populate with verticies that are then drawn in the java context using a GLSurfaceView.

I decided to simply tick the word inside of the UI draw thread of the surface view. You are not supposed to do expensive computation in the UI thread, and thats true for the most part in this case. Ideally your phone is fast enough that the word can be simulated at the rate at which the surface view is drawn which is normally 60fps on phones.


# Author notes
As I delved further and further into this passion project. I came to realize that I may have "bitten off more than I could chew" given my life responsibilities. So this isn't really as "rigorous" as I would like it to be.


## License

Licensed under the terms of MIT license and the Apache License (Version 2.0).

See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE) for details.
