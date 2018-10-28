






In this document, we'll go over the high level design of some of the algorithms provided by this crate.
As a user of this crate, you don't really need to know any of these. All you need to concern yourself about is the users guide.

# Tree data structure

First lets talk about the tree data structure itself before we talking about the algorithms that exploit its properties. Early in development, from benching some of the early query algorithms, it was apparent that the bottleneck was the querying of the tree, not the construction of the tree. So right off the bat, the design desision was made to make the tree such that once it was constructed, querying was fast as possible even if this meant comprimizing on construction performance. So memory compactness is the key goal.

## Construction

Construction works as follows, Given: a list of bots.

For every node we do the following:
	1) First we find the median of the remaining bots (using pattern defeating quick sort) and use its position as this nodes divider.
	2) Then we bin the bots into three bins. Those strictly to the left of the divider, those strictly to the right, and those that intersect.
	3) Then we sort the bots that intersect the divider along the opposite axis that was used to finding the median.
	4) Now this node is fully set up. recurse left and right with the bots that were binned left and right. This can be done in parallel.

## Memory Layout

Memory layout is extremely important when you have an algorithm that has an inner loop over a big data structure. (for example intersect pair finding). So our goal is to layout the tree in memory so that it is very compact, and also that when you visit all the nodes, you do so in such a way that memory locality is promoted. 

### Memory compactness

A typical implementation might work this way. Every node contains data about the aabb of that node, where the divider is, and then it would also contain a reference to slice of bots in a vec somewhere else in memory. The problem with this layout is that the vec of bots and the vec of nodes are allocated seperately and while they have good locality between the elements in each other, they dont have good locality between each other. The solution is to not have two seperate vecs, but to have just one vec where every node literally has a slice of bots - not a reference to a slice of bots, but the slice itself. Every node is therefore a dynamically sized type. So the natural problem here is that every node is going to have a different number of bots in it. So every node will have a different size. You cant have a Vec of these kinds of types since a vec assumes all the elements have the same size! The solution is manual memory management. Knowing the number of bots and the number of nodes upfront, we can calculate exactly how much memory we need to allocate. Then we can keep a counter as we insert each node. Nodes are connected via pointers to their children.

### Memory Locality

So that should give you an idea of how we achieve great memory compactness, but what about memory locality? Well lets think about what kind of pattern algorithms that work on trees tend to work. The main primitive that they use is accessing the two children nodes from a parent node. Fundamentally, if I have a node, I would hope that the two children nodes are somewhat nearby to where the node I am currently at is. More importantly, the further down the tree I go, I would hope this is more and more so the case! For example, the closeness of the children nodes to the root isnt that important since there is only one of those. On the other hand, the closeness of the children nodes for the nodes at the 5th level of the tree are more important since that are 32 of them. 

So how can we layout the nodes in memory to achieve this? Well, putting them in memory in breadth first order doesnt cut it. This achieves the exact opposite. For example, the children of the root are literally right next to it. On the other hand the children of the most left node on the 5th level only show up after you look over all the other nodes at the 5th level. It turns out in-order depth first search gives us the properties that we want. With this ordering, all the parents of leaf nodes are literally right next to them in memory. The children of the root node are potentially extremely far apart, but that is okay since there is only one of them.

### Destruction of tree

If we were inserting references into the tree, then the original order of the bots is preserved during construction/destruction of the tree. However, we are inserting the actual bots to remove this layer of indirection. So when we destroy the tree, we want to return the bots to the user is the same order that they were put in. This way the user can rely on indicies for other algorithms to uniquely identify a bot. To do this, during tree construction, we also build up a Vec of offsets to be used to return the bots to their original position. We keep this as a seperate data structure as it will only be used on destruction of the tree. If we were to put the offset data into the tree itself, it would be wasted space and would degrade the memory localicty of the tree query algorithms. We only need to use these offset once, during destruction. It shouldnt be the case that all querying algorithms that might be performed on the tree suffer performance for this.

### Memory complexity during construction

Construction involves a couple of allocations. TODO talk
The user has a vec a bots. Then a vec of aabb's and offsets is generated from this vec of bots. This vec is then sorted and binned into a dinotree. So at this point we have the original vec of bots, and a tree of aabb's and offsets. These two vecs are then melded together into one dinotree. So we end up need memory space for 2*n+(2*n) because we need:
1) space for all the bots
2) space for all the aabbs and offsets (well assume the size of a bot is atleast as big as this)
3) space for the fused together tree which is a combination of the two above.
So thats n+n+2*n=4*n memory space.
So its a fair about of memory space needed, but at least it grows linear.


### Leaves

The leaves do not have dividers. This means that if we use the same type for both nonleaves, and leaves we have wasted space in our leaf objects. This wouldnt be so bad if it wernt for two things. First, this is a complete binary tree, so literally half the nodes are leaves. Second, before the tree is laid out in dfs in order order in memory, this means that the distance between the root node and its children is effected by the empty space all of the leaves between them of which a quarter of the leaves will be. Our goal is to make this as compact in memory as possible, so to avoid this, we simply have two different types. 

### Knowing the axis as compile time.

A problem with using recursion on an kd tree is that every time you recurse, you have to access a different axis, so you might have branches in your code. A branch predictor might have problems seeing the pattern that the axis alternate with each call. One way to avoid this would be to handle the nodes in bfs order. Then you only have to alternate the axis once every level. But this way you lose the nice divide and conquer aspect of splitting the problem into two and handling those two problems concurrently. So to avoid, this, the axis of a particular recursive call is known at compile time. Each recursive call, will call the next with the next axis type. This way all branching based off of the axis is known at compile time. A downside to this is that the starting axis of the tree
must be chosen at compile time. It is certainly possible to create a wrapper around two specialized versions of the tree, one for each axis, but this would leads to alot of generated code, for I suspect little benefit. Not much time is spent handling the root node anyway, so even if the suboptimal starting axis is picked it is not that big of a deal.



# Algorithms overview

Now that we've estblished the properties of the tree, lets talk about what we can do with it. 
All these algorithms use the tree provide by the dinotree_inner crate, although some do not fully exploit all properties of this tree. (nbody does not take advantage of the fact that the bots in a non leaf node are sorted, for example)

# Finding all intersecting pairs

Done via divide and conquer. For every node we do the following:
1) First we find all intersections with bots in that node using sweep and prune..
2) We recurse left and right finding all bots that intersect with bots in the node.
	Here we can quickly rule out entire nodes and their decendants if a node's aabb does not intersect
	with this nodes aabb.
3) At this point the bots in this node have been completely handled. We can safely move on to the children nodes 
   and treat them as two entirely seperate trees. Since these are two completely disjoint trees, they can be handling in
   parallel.

## Using sweep and prune internally

The sweep and prune algorithm is a good candidate to use since, for one thing is uses very little memory (just a stack that can be reused as you handle decendant nodes). But the real reason why it is good is the fact that the bots are likely to be stewn across a line. Sweep and prune degenerates when the active list that it must maintain has many bots that end up not intersecting. This isnt likely to happen for the bots that belong to a node. The bots that belong to a non leaf node are guarenteed to touch the divider. If the divider partitions bots based off their x value, then the bots that belong to that node will all have x values that are roughly close together (they must intersect divider), but they y values can be vastly different (all the bots will be scattered up and down the dividing line). So when we do sweep and prune, it is important that we sweep and prune along axis that is different from the axis along which the divider is partitioning.

## Performance

The construction of the tree may seem expensive, but it is still less than the possible cost of this algorithm. This algorithm could dominate very easily depending on how many bots intersect. That is why the cost of sorting the bots in each node is worth it because our goal is to make this algorithm the fasted it possibly can be. The load of the rebalancing of the tree doesnt very as much as the load of this algorithm. 


# Nbody

The nbody algorithm works in three steps. First a new version tree is built with extra data for each node. Then the tree is traversed taking advantage of this data. Then the tree is traversed again applying the changes made to the extra data from the construction in the first step.

The extra data that is stored in the tree is the sum of the masses of all the bots in that node and all the bots under it. The idea is that if two nodes are sufficiently far away from one another, they can be treated as a single body of mass.

So once the extra data is setup, for every node we do the following:
	Gravitate all the bots with each other that belong to this node.
	Recurse left and right gravitating all bots encountered with all the bots in this node.
		Here once we reach nodes that are sufficiently far away, we instead gravitate the node's extra data with this node's extra data, and at this point we can stop recursing.
	At this point it might appear we are done handling this node the problem has been reduced to two smaller ones, but we are not done yet. We additoinally have to gravitate all the bots on the left of this node with all the bots on the right of this node.
    For all nodes encountered while recursing the left side,
    	Recurse the right side, and handle all bots with all the bots on the left node.
    	If a node is suffeciently far away, treat it as a node mass instead and we can stop recursing.
    At this point we can safely exclude this node and handle the children and completely independent problems.



# Raycasting


TODO explain

# Knearest

TODO explain


# Rect

TODO explain.
