Segment RTree
=============
This introduces and developes a _Segment RTree_, a version of an RTree for
LineStrings uses the segments in order as leaf nodes of the RTree.  
Each subtree to represent a contiguous section of the LineString,
which allows these sections to be processed at once in certain algorithms.

Included are algorithms for point-in-polygon, LineString validity, and
Rectangle-LineString clipping, and hopefully soon will include Polygon
validity, Rectangle-Polygon clipping, and eventually overlay operations.

I've started on some initial benchmarking, and a Flatbush implementation to
compare against.  It'd be great to have an STR RTree as well.

This is a proof-of-concept/WIP that is very rough and probably has lots of
bugs.

There is currently no friendly license for this code, not due to malice
but rather incompleteness.  I expect to eventually release it freely.