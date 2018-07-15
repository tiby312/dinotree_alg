


# Testing


## Testing Correctness

Compare against naive

## Testing Memory layout


## Testing 


#
It's important that if we write a test over n, we dont not inadvertatly influence other variables.
As n grows, if we keep adding bots to the same enclosed space, then they will overlap more.
What we want to test is what happens as n grows, and density stays roughly the same.
I tried to come up with a formula to calculate the dimensions needed for given desntiy and number of bots,
with a random distribution, but couldnt come up with it. If some one could help that would be great.
Instead, I used a simple spiral. It grows from the center, adding bots as it goes.





# Testing correctness

Simply using rust has a big impact on testing. Because of its heavy use of static typing, many bugs are caught at compile time. This translates to less testing as there are fewer possible paths that the produced program can take. Also the fact that the api is generic over the underlying number type used is useful. This means that we can test the system using integers and we can expect it to work for floats. It is easier to test with integers since we can more easily construct specific scenarios where one number is one value more or less than another.

A good test is a test that tests with good certainty that a large portion of code is working properly.
Maintaining tests comes at the cost of anchoring down the design of the production code in addition to having to be maintained themselves. As a result, making good abstractions between your crates and modules that have very simple and well defined apis is very important. Then you can have a few simple tests to fully excersise an api and verify large amounts of code.

This crate's sole purpose is to provide a method of providing collision detection. So a good high level test would be to compare the query results from using this crate to the naive method (which is much easier to verify is correct). This one test can be performed on many different inputs of lists of bots to try to expose any corner cases. So this one test when fed with both random and specifically tailed inputs to expose corner cases, can show with a lot of certainty that the crate is satisfying the api. 

The tailored inputs is important. For example, a case where two bounding boxes collide but only at the corner is an extremely unlikely case that may never present themselves in random inputs. To test this case, we have to turn to more point-directed tests with specific constructed set up input bot lists. They can still be verified in the same manner, though.

# Benching

Writing benches that validate every single piece of the algorithm design is a hassle ,although it would be nice. Ideally you dont want to rely on my word to say that, for example, using sweep and prune to find colliding pairs actually sped things up. It could be that while the algorithm is correct and fast that this particular aspect of the algorithm actually slows things down. 

So I dont think writing tons of low level benches are worth it. If you are unsure of a piece of code, you can bench the algorithm as a whole, change a piece of the algorithm, and bench again and compare results. Because at the end of the day, we already tested the correctness, and that is the most important thing. So I think this strategy, coupled with code coverage and just general reasoning of the code can supplement tons of benches to validate the innards of the algorithm.



One thing to notice. Collision detection is expensive, and dominates the running time. So right off the bat, you know it is a waste of effort to allocate time into optimizing the linear parts of your program, when you could be spending that time optimizing the bottleneck.

Always measure code before investing time in optimizing. As you design your program. You form in your mind ideas of what you think the bottle necks in your code are. When you actually measure your program, your huntches can be wildly off.

Dynamic allocation is fast. Dynamically allocating large vecs in one allocation is fast. Its only when you're dynamically allocting thousands of small objects does it become bad. Even then, probably the allocations are fast, but because the memory will likely be fragmented, iterating over a list of those objects could be very slow. Concepually, I have to remind myself that if you dynamically allocate a giant block, its simply reserving that area in memory. There isnt any expensive zeroing out of all that memory unless you want to do it. That's not to say the complicated algorithms the allocator has to do arn't complicated, but still relatively cheap.

Often times, its not the dynamic allocation that is slow, but some other residial of doing it in the first place. For example, dynamically allocating a bunch of pointers to an array, and then sorting that list of pointers. The allocation is fast, its just that there is no cache coherency. Two pointers in your list of pointers could very easily point to memory locations that are very far apart.


The thing is that if you don't use dynamic allocation, and you instead reserve some giant piece of memory for use of your system, then that memory is not taken advanage of when your system is not using it. It is wasted space. If you know your system will always be using it then sure it is fine. But I can see this system being used sometimes only 30 times a second. That is a lot of inbetween time where that memory that cannot be used by anything else. So really, the idea of dynamic allocation only works is everybody buys into the system. Another option is to make your api flexible enough that you pass is a slice of "workspace" memory, so that the user can decide whether to dynamically allocate it everytime or whatever. But this complicates the api for a very small portion of users who would want to not using the standard allocator.

When dealing with parallelism, benching small units can give you a warped sense of performance. Onces the units are combined, there may be more contention for work stealing. With small units, you have a false sense that the cpu's are not busy doing other things. For example, I parallalized creating the container range for each node. Benchmarks showed that it was faster. But when I benched the rebalancing as a whole, it was slower with the parallel container creation. So in this way, benching small units isnt quite as useful as testing small units is. That said, if you know that your code doesnt depend on some global resource like a threadpool, then benching small units is great.

Platform dependance. Rust is a great language that strives for platform independant code. But at the end of the day, even though rust programs will behave the same on multiple platforms, their performance might be wildly different. And as soon as you start optimizing for one platform you have to wonder whether or not you are actually de-optimizing for another platform. For example, rebalancing is much slower on my android phone than querying. On my dell xps laptop, querying is the bottle neck instead. I have wondered why there is this disconnect. I think part of it is that rebalancing requires a lot of sorting, and sorting is something where it is hard to predict branches. So my laptop probably has a superior branch predictor. Another possible reason is memory writing. Rebalancing involves a lot of swapping, whereas querying does involve in any major writing to memory outside of what the user decides to do for each colliding pair. In any case, my end goal in creating this algorithm was to make the querying as fast as possible so as to get the most consistent performance regardless of how many bots were colliding.

In fact, I ended up with 3 competing rebalancing algorithms. The first one would simply create pointers to all the bots, and sorted the pointers. This one was the slowest. I think it is because only one field is relevant to this algorithm, the bounding box rect. So all the other fields were just creating space that needed to be jumped over. So the distance between relevant information to be used by the algotihm wa high. On the other hand this method didnt have to allocate much memory. There is also the problem that its highly dependant on where the given slice is in memory. If its far away from the vec of pointers, then every deref is expensive, probably.

The second one would create a list of rects and ids pulled from the bots and sort that. The main characteristic of this method is that there is no layer of indirection. The downside is that swapping elements is more expensive since you are not swapping pointers, you are swapping bounding boxes coupled with an id. So this method made the median-finding part of the algorithm very fast, but made the sorting slower.

The third method was to create a list of rects, and then create a list of pointers to that list of rects and then sort that. The obvious downside is that you end up dynamically allocating two seperate vecs. But really it doesnt use any more memory that method 2. It has the benefit of swapping only pointers, and it also has better memory locality that method1.

For large numbers of bots 50,000+, the second method seems to be the best on both my phone and laptop.

