
#prefer NotNaN<> over OrderedFloat<>.
NotNaN has no overhead for comparisions, but has overhead for computation, the opposite is true for OrderedFloat.
For querying the tree colliding pairs, since no calculations are done, just a whole lot of comparisions, prefer NotNaN<>.
Other queries do require arithmatic, like raycast and knearest. So in those cases maybe ordered float is preferred.


#
Ideally you only construct the bounding box for the duration that you use the tree.
This way, the algorithms that dont use the bounding box have better spatial locality in memory.
