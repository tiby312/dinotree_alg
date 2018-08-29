
Collision finding is the only one that actually takes advantage of the sorted property.



#
let X be the random variable destrbing whether or not two bots collide.
Becuase the two bots are indenentantly placed in the word uniformly, we can say:
P(X)=(bot_width/dim_x)^2*(bot_height/dim_y)^2
Because X only has the values true or false, E[X]=P(X).

let Y be the random variable describing the number of bots colliding.
So Y=X1+X2+X3..Xn where n is the number of pairs of bots.
Or Y=X*(n choose 2) where n is the number of bots.
Each of these events is independant, and identical to each other.
Exploiting the linearity property of the expected value, 
E[Y]=E[X*(n choose 2)]=(n choose 2)*E[X]

What we want to find is the average number of bots colliding for a particular point
in the 2d space (lets call this L). So we simply divide the number of bots colliding by the area.
E[L]=E[Y]/(dim_x*dim_y)

E[L]=(n choose 2)*E[X]/(dim_x*dim_y)
E[L]=(n choose 2)*( (bot_width/dim_x)^2*(bot_height/dim_y)^2 ) /( dim_x*dim_y)
Simplifying this much further is hard.
Let as make an assumption that the word and the bots are square.
E[L]=(n choose 2)*( (bot_width/dim_x)^2*(bot_width/dim_x)^2 ) /(dim_x*dim_x)
E[L]=(n choose 2)*( (bot_width/dim_x)^4 ) /(dim_x^2)
E[L]=(n choose 2)* bot_width^4 / dim_x^2

So now if we fix any of the 3 variabes, we can calculate the third.
Lets solve for the dim.

dim_x^2=(n choose 2)* bot_width^4 /E[L]
dim_x=sqrt((n choose 2)* bot_width^4 /E[L])


 Now lets sanity check.
 If we plug in:
 n=100*100.
 bot_width=1;
 E[L]=1;
 we should get 100 back.

# Space and Time Complexity

I dont what the theoretical average time compleity of this algorithm would be. The performance depends so wildly on the distribution of the position and sizes of the bots that are fed into it. And in more usecases, there would be certain patterns to the input data. For example, in most cases, I would hope that the bots are mostly not intersecting, (because presumably the user is using this system to keep the bots apart). And another presumption might be that size of the bounding boxes would tend to be small relative to the world in which all the bots live. 

In the average case, if you thought of all the bots as nodes, and then added edges to the nodes whose bots intersected, you'd hope that your graph was planar. This might be another way of figuring out the time complexity. The number of edges of a planar graph is bounded from above by 3*v-6. This is much smaller than the naive v*v edges of a complete graph.

That said bounding it by the worst case is easy, because in the worst case every single bot is colliding with every other bot. So the worst case is that all the bots are directly ontop of each other. Then the tree nor the mark and sweep algorithm could take any adantage of the situation and it would degenerate into the naive algorithm.

In the best case, all the bots live in only leaf nodes, and none of the bots intersect. Interestingly by the pigeon principle, if you have more bots than there are leaf nodes then this best case scenario isnt possible. And this is the case. We are picking the height of the tree such that every leaf node will have a specific amount of bots. We also know that every non leaf node will most likely have at least one bot in it since it was used as the median. The non leaf nodes that dont have any bots in them, must not have any because none of its children have bots either.

# Epsilon

Before we analyze the rebalance and query algorithms, lets come up with an approximation of how often bots would intersect a divider. Lets first look at the root. If you had a bunch of bots randomly and uniformly distrubuted in a 2d space, how many of them would intersect the median divider? The answer to this depends on the sizes of the bots. If all the bots were points, then hopefully only one bot would intersect with the divider. The only case this wouldnt be true is if multiple bots had the same x position as the median bot. If we're talking about real numbers, then I think the likelyhood of two bots randomly sharing the exact same x value is next to impossible. Since we are not dealing with real numbers, its more likely. On some bounded interval, there are only so many values that a floating point can have inbetween them, and even less so for integers. But it would still be a small enough chance that we can ignore. So for the cases where the bot is a point, I think its safe to say that epsilon is around 1 for the root.

As the sizes of the bots increases, epsilon would also grow. By how much I'm not sure. But thats not the real concern. We are only concerned about the complexity as n grows. We can just assume that the bot size is constant, whatever it may be. 
For our purposes, its simpler to just think of the bots as points since it doesnt effect our n complexity.

So the question is as n grows, how is episolon effected?

It clearly must also grow somewhat. The more bots there are, the greater the likelyhood that any bot will have the same value as the median bot.  
So we have:
1/x + 1/x +1/x +1/x + ... =  n/x
where x is the possible x values.


probability 1 bot intersects median:p(bot)=d/s
let random variable be whether or not bot touches x. So it either is or it isnt.
It happens with probability d/s. It doesnt happen with probably 1-d/s.
So E[X]=1*d/s+0*(1-d/s)=d/s.

Expected value is additive. so E[x1+x2+..]=E[x1]+E[x2]+E[x3]...

so we have: (nd)/s=expected value of each bot touching.

So that is just for the root. For the other levels, I couldn't come up with a nice closed form equation. Just recursve ones that depend on the acencetors. So lets just assume that each level is just as bad as the root. In reality, each level would have less bots to consider. 

So we're saying that for any level, we expect  (n*d)/s to intersect the divider.

Again we are assuming uniform distribution.

So below algorithms are only efficient if d<<s. 










# Rebalance Algorithm time complexity

Lets looking at rebalancing. For each node, there are three steps that need to be done:
binning process
sort middile
recurse left,right

As you go down the tree, less time is spent binning, and more time is spent sorting.

at the root, binning would take N, and sorting would take epsilon (the amount intersecting the divider. The hope is that this is  asmall number).
at the second level, binning would be (N-e1), and sorting would take 2*e2, so if we write this out:


level1  =  1*bin(n)+sort(e)

level2  =  2*(bin((n-1e)/2)+sort(e/2))

level3  =  4*(bin((n-2e)/4)+sort(e/4)) 

level4  =  8*(bin((n-4e)/8)+sort(e/8))






(bn-se1)+2*se2
bn-(se1+2*se2)+4*se3
...
Lets make further assumption that all e's are roughly the same.
bn+se
bn-se+2*se=bn+se
bn-(se+2*se)+4*se=bn-3se+4se=bn+se
...

so I think each level would take b(n)+s(e).
The number of levels is log2(n/10);
So in total: (bin(n)+sort(e))*log2(n/10);

So we have: (bin(n)+sort((n*d)/s))*log2(n/10);

binning grows linearly, but sorting (via rusts std sort) grows by n*log(n).

So as far as number of steps and ignoring constants we have:
complexity(n)=(n+n*log2(n))*log2(n); where x=n*d/s;
n*log2(n)+n*log2(n)*log2(n)<=2*nlog2(n)*nlog2(n)==nlog2()^2

So I think complexity of rebalancing is nLog2(n)^2. But in practice is d<<s, it is probably closer to nlog(n).




Querying on the other hand is more challening, lets give it a shot:
Lets make some sweeping (no pun intended) assumptions. Every node has around the same number of bots,
and we will call it e (same as rebalancing)

Level 1: from the root we have to recurse all the way down until we visit all nodes that touch the root divider.
	sweep(e)+bjsweep(e)*h
level 2:
	sweep(e)*2+bjsweep(e)*(h-1)*2
level 3:
	sweep(e)*4+bjsweep(e)*(h-2)*4

so we have:

(se+be*h) + 2*(se+be*(h-1)) + 2^2(se+be*(h-2)) + ...

Lets split it into two terms

(se+2*se+4*se+....)+(be*h  + 2*be*(h-1)+4*be(h-2)+..)

now lets distribute:

se*(2^0+2^1+2^2+2^3+...)   + be*(1h +2(h-1)+4(h-2)+8(h-3)+...)
                                 


The first term is a geometric series, so is equal to:
se*(2^h-1)
or roughly:
se*(2^h)

The second term, is more complicated, but a geometric series can be broken off and you are left with a summation
over ia^i. After some simplifying the second term is close to:
be(h*2^h)

I'm dropping small constants left and right since we only care about the complexity at a large scale.

so we have:
se*(2^h)+be(h*2^h)

here:
2^h(se+be*h);

We want to bound it by a function that takes n as input, not h.

2^(log2(n/10))*(se+be*log2(n/10))

(n/10)(se+be*log2(n/10))

So I think the complexity of the querying is also O(n*log2(n)), but it is clearly more expensive than rebalancing.



So overall we have to functions that bound complexity:

rebalance_cost=(bin(n)+sort(e))*log2(n/bots_per_node);
query_cost=(n/bots_per_node)(sweep_sort(e)+bi_sweep(e)*log2(n/bots_per_node))

So things to notice about these functions. The bin function takes the entirety of n as input. Even though binning is fast (only have to put the bots into 3 buckets), it has to be applied to a large number of bots. By contract, the sorting function is done on on a smaller epsilon.

The bijective sweep algorithm is less expensive than the sweep sort. The sweep sort algorithm has to check every body against every other in the slice that is passed to it. By contrast, the bi_sweep() algorithm checks two slices of bots only against each other. 










The space complexity, on the other hand, is much easier to figure out. 
The height of the tree is=log2(num/10).
The num of nodes=2^height.
So the number of nodes as a function of bots is nodes=2^(log2(num/10))=num/10.
So the number of nodes created is linear to the number of bots.
So I think space complexity is O(n).
