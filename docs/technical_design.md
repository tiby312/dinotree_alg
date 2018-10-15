The main point is that no arithmatic is done. Use must supply this.


## Continuous Collision detection
TODO

## Dynamic Allocation

Sure dynamic allocation is "slow", but that mean you should avoid it. You should use it as a tool to speed up a program. It has a cost, but the idea is that with that allocated memory you can get more performance gains.

The problem is that everybody has to buy into the system for it to work. Anybody who allocated a bunch of memory and doesn't return it because they want to avoid allocating it again is not hogging that space for longer than it needs it.

Writing apis that don't do dynamic allocation is tough and can be cumbursome., since you probably have to have the user give you a slice of a certain size. On the other hand this let's the user know exactly how much memory your crate needs.




Parallalization is done using rayon. The rust slice provided split_at_mut() and rayon's join() are two extremely powerful constructs. Seeing and understanding the api and implementation of split_at_mut() is what convinced me that rust was the future.
At a certain point while going down the tree, we switch to a sequential version as recommended by rayon's usage guidelines when used in recursive divide and conquer problems. The depth at which point we switch to sequential should not be static. It should change along with the size of the problem. TODO explain this more.



# General thoughts on optimizing


Optimization is a great balancing act. There are so many interesting questions. Indirection, locality, memory vs computation, dividng and conquering. Every algorithm has unique properties. On top of the designing the algorithm, there is a whole nother level of interesting questions when it comes to implementing said algorithm. How to best write the code for maintainability, readabilty, performance, simplicity in the api. Concurrency is the strangest of all. The theory and in practice clash together. I think coding is well suited for people who like chess. The same interesting desision making takes place. "big picture" thinking is very important, and you are often rewarded for following your intuition and hunches down some path. Making desisions of when to stabailize and write tests, or when to capitalize on some new design oportunity, it goes on. Is it better to generate out less code with more branches, or more code with less branches.  At one point, I wanted to avoid dynamically checking the axis at runtime, so I made a axis trait, and each level of the recusion would call the next level with that axis trait's associated type which indicated the alternate axis. The problem with this is that even though you are saving checking this axis, you have inflated your code to be very big. Any gains from lessening the branching, was added back for more code that your program had to preocess.


#mutability and aabb
none of these algorithms allow the user to modify the aabb inside of the callback functions. sure the type system allows it, (you have a mutable reference to the bots inside of the callbacks), but part of the contract of implementing the HasAabb trait is that you will not modify the aabb in these callbacks. Once a dinotree is contructed the aabbs dont move. Creating algorithms that allow you do change the aabb would require a lot of shifting. removing/inserting bots would invalidate the tree pointers since everything is tightly packed in memory. This is just not the data structure for that kind of behavior.



#colfind
I am not convinced that storing points+use supplied left,top,right,bottom border retrieval functions is faster than just
storing the aabb. This strategy would use less memory. (No need to sore the aabb, just the center point), But the aabb ends up being calculated from the point. I'm doubtful that this is faster because many floating point operations would be necessary. And what is more
is that many of these floating point operations and computing the same thing over and over again. During the colfinding,
the aabb for a bot might need to be checks a bunch of times. Thats a lot of extra floating point operations. So I thin having the computed aabb in memory is better. 
The other downside is that you lose generality. All the aabb's must be the same size.




# Exploiting temporal locality

There's really two different contexts in which temporal locality can be talked about. There's the time locality between states of the 2d world between calls to create and destroy the tree. Then there's the time locality of the internal locality as it makes it ways through the alogrithm.

The sort answer? It does not. For a while I had the design where the dividers would move as those they had mass. They would gently be pushed to which ever side had more bots. The problem with this approach is that the divider locations will mostly of the time be sub optimial. And the cost saved in rebalancing just isnt enough for the cost added to querying with a suboptimal partitioning. By always partitioning optimally, we get guarentees of the maximum number of bots in a node. Remember querying is the bottleneck, not rebalancing.




Another strategy to exploit temporal locality is by inserting looser bounding boxes into the tree and caching the results of a query for longer than one step. The upside to this is that you only have to build and query the tree every couple of iterations. There are a number of downsides, though:

* Your system performance now depends on the speed of the bots. The faster your bots move, the bigger their loose bounding boxes, the slower the querying becomes. This isnt a big deal considering the ammount that a bot moves between two frames is expected to be extremely small. But still, there are some corner cases where performance would deteriorate. For example, if every bot was going so fast it would just from one end of you screen to the other between world steps. So you would need to bound the velocity of your bots to a small value.

* You have to implement all the useful geometry tree functions all over again, or you can only use the useful geometry functions at the key world steps where the tree actually is constructed. For example, if you want to query a rectanlge area, the tree provides a nice function to do this, but you only have the tree every couple of iterations. The result is that you have to somehow implement a way to query all the bots in the rectangle area using your cached lists of colliding bots, or simply only query on the world steps in which you do have the built tree. Those queries will also be slower since you are working on a tree with loose boxes.

* The maximum load on a world step is greater. Sure amortised, this caching system may computation, but the times you do construct and query the tree, you are doing so with loose bounding boxes. On top of that, while querying, you also have to build up a seperate data structure that caches the colliding pairs you find. 

* The api of the dinotree is flexible enough that you can implement loose bounding box + caching on top of it (without sacrificing parallelism) if desired.



So in short, this system doesnt take advantage of temporal locality, but the user can still take advantage of it by inserting loose bounding boxes and then querying less frequently to amortize the cost. I didnt explore this since I need to construct the tree every iteration anyway in my android demo, because I wanted the feedback of the user moving his finger around to be imeddiate. So to find all the bots touching the finger i need the tree to be up to date every single iteration. This is because I have no way of know where the user is going to put his finger down. I cant bound it by velocity or acceleration or anything. If I were to bound the touches "velocity", it would feel more slugish i think. It would also delay the user putting a new touch down for one iteration possibly.



# Extensions and Improvements

What about 3d? Making this multi dimensional would have added to the complexity, so the design desision was made to only target 2d. Its much easier for me as a developer to visualize 2d. So as a good first iteration of this algorithm, targeting just 2d simplifies things. Expanding it to 3d, shouldnt take too much effort. The hard part would be over. Code architecture would hopefully not need to be changed much.  


That's not to say one couldn't still take advantage of this system in a 3d simulation. Every bot could store a height field that you do an extra check against in the collision function. The downside is that imagine if there were many bots stacked on top of each other, but you only wanted to query a small cube. Then doing it this way, your query function would have to consider all those bots that were stacked. If there are only a few different height values, one could maintain a seperte 2d dinotree for each level. Looking at the real world though, and most usecases, your potential z values are much less than our potetial x and y values. So for many cases, it probably better to use the tree for 2 dimentions, and then naively handling the 3rd. Then you dont suffer from the "curse of dimensionality"?

Pipelining. It might be possible to pipeline the process so that rebalancing and querying happen at the same time with the only downside being that bots react to their collisions one step later.

Another possible "improvement" would be to store positions and radius instead of bounding boxes.
That saves one extra float, but it is less versatile. Also fixing the radius of the bots would be a
huge performance improvement. Every bot would only need to store a position then.



# Use of Unsafe

moving objects that dont implement copy.
The multirect example. 
split_at_mut()

# think about it like a sponge. the lower you go into the tree, the more stable the calculates get 
