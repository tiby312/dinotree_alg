
#prefer NotNaN<> over OrderedFloat<>.
NotNaN has no overhead for comparisions, but has overhead for computation, the opposite is true for OrderedFloat.
For querying the tree colliding pairs, since no calculations are done, just a whole lot of comparisions, prefer NotNaN<>.
Other queries do require arithmatic, like raycast and knearest. So in those cases maybe ordered float is preferred.


#
Ideally you only construct the bounding box for the duration that you use the tree.
This way, the algorithms that dont use the bounding box have better spatial locality in memory.




#user responsibilty
Its always the goal to make the objects that are put into the tree be as small in size as possible.
To do this, every member in each object should actually be used during the querying.
If there is a member that isnt used, you will improve memory locality, by making a new type
with less memory, construct and query the tree with that, and then re-inject changes into the original object.
For example, if your colliding pair function pushes pairs of bots apart purely using pos+radius, and the velocity field is
never used in the colliding function, then construct the tree with a new type without velocity. Another common example
of a field that might not be needed is an id field. 

For this same reason, I dont think a special algorithm for findind col pairs with an extended raidus around each aabb is worth it.
It would end up requiring more bounds calculations. For these cases, I think simply making aabb's big enough to capture that extra radius is faster.
