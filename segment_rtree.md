Segment RTree
=============

A segment RTree is an RTree for a LineString, whose leaf nodes are the envelopes
of the segments of the LineString, in order.  This means that the parents and
grandparents of the leaf nodes represent blocks of contiguous segments.  Thus,
when descending into the Segment RTree, one is discarding/accepting hierarchically
nested sub-LineStrings.

While this is generally not an optimal clustering of an RTree, it's generally
reasonable since we are guaranteed that adjacent segments are proximate (in fact
intersecting), and it's a reasonable prior that segments proximate in "index space"
are proximate in geometry.  Also, since there is no re-ordering, the Segment
RTree is very fast to build, particularly if one uses a packed representation.

With this assumption, the Segment RTree retains more information about the
structure of the LineString, which we can exploit for algorithms. In
particular: each node has two indices `low` and `high` that describe the
sub-LineString represented by its sub-RTree.  Notice:

1. Leaf nodes have `high == low + 1`.
2. For parent nodes, `low` and `high` are the minimum/maxiumum of those of
its children.
3. The root node has `low == 0` and `high == coordinates.length - 1`.

Also notice that we can derive these indices from the degree (or branching
ratio) of the tree, the level of the node, and the index of the node on its
level.  Thus we don't need to explicitly store them.

Point-in-Polygon
================
Consider the [winding-number test][winding-number-test] for point-in-polygon
containment. In this, we calculate the winding number of a LinearRing around
a point by iterating through each segment, and adding 0, +1, or -1 based on
how the segment intersects (or doesn't) a ray from the point to `x ==
+infinity`. Without preparation, this takes `O(V)` time, where `V` is the
number of vertices in the ring. Note that for a polygon with holes, one can
either iterate through the segments of the shell and all holes, or check for
containment in the shell and non-containment in each hole. Note that this
procedure can also determine if the point is on a boundary of one of the
rings, also a condition of interest.

Consider a polygon with no holes, represented by a single LinearRing. If an
RTree of the segments of the ring is prepared (`O(V * log V)`), we can query
the RTree with the envelope of the ray, only checking segments that are to
the right and traverse the `y` value of the point. This brings down the check
to `O(log V)` on average, although there are many cases where this fails.
Consider a polygon, like Russia's border, that has a long complex horizontal
section. A point near the western part of that border potentially has to
check a reasonable fraction of all segments of the polygon, bringing the
complexity closer to `O(V)`.

We will later prove a claim about Segment RTrees that will make this
significantly cheaper:

**Claim 1**: If a node is to the right of a point, then the sum of the winding
numbers of its leaves is equal to the winding number of the segment
`{start: coordinates[node.low], end: coordinates[node.high]}`.

With this claim, when we are recursing down the Segment RTree for the
containment check:
1. Discard any node above, below, or to the left of the point, and
2. Any node to the right (`node.x_min > p.x`), calculate the winding number of
the "node segment" (`O(1)`), and
3. Recurse into the children of any node that contains the point.

This makes checks like the Sibera case above much cheaper, since many of the
segments to the right of the point will be included in a higher node of the
RTree.

**Proof of Claim 1**: Since the LineString has no self-intersections,
the path between `start` and `end` can be continuously deformed into the
straight line between `start` and `end`. Since the winding number is a
topological invariant (the homotopy of the map `ring -> S1`), it is unchanged
by this deformation.

For a more constructive proof, conside two adjacent segments, `q1` to `q2`
and `q2` to `q3`. Let `wn12 = winding_number(q1, q2)`, and similarly for
`wn23` and `wn13`. If `wn13 = wn12 + wn23`, then we can successively "delete"
intermediate vertices until we have the straight-line segment. This equality
can be checked explicitly with the [winding number
formula][winding-number-test].

Algorithm for Segment RTree point-in-polygon check:

```
def point_in_polygon(p: Point, rtree: SegmentRTree, coords: List[Coordinate]):
    wn = 0
    stack = []
    stack.push(rtree.root)
    while not stack.is_empty():
        node = stack.pop()
        if node.y_min > p.y or node.y_max < p.y or node.x_max < p.x:
            continue
        else if node.x_min > p.x or node.is_leaf():
            start = coords[node.low]
            end = coords[node.high]
            wn += winding_number(p, start, end)
        else:
            for child in node.children:
                stack.push(child)
    return wn != 0
```


LineString Validity
-------------------
If a LineString has a prepared RTree (including Segment RTree), it can quickly
find self-intersections.  The algorithm is roughly:
```
def query_self_intersections(rtree: RTree):
    stack = []
    stack.push((rtree.root, rtree.root))
    while not stack.is_empty():
        left, right = stack.pop()
        if not left.intersects.right():
            continue
        if left.is_leaf() and right.is_leaf():
            if left.index < right.index:
            yield left, right
            continue
        for left_child, right_child in left.children.product(right.children):
            if left == right and left_child.index > right_child.index:
                continue
            stack.push((left.child, right_child))
```

Using the indices of the left and right node, the actual intersection check
can be made.  In the case of LineString segments, you must also check for the
valid intersections of a segment with its adjacents (including first and last
segments for LinearRings).

Constructing the RTree takes `O(V * log V)` time, and the above algorithm
takes on average `O(N + I)`, where `N` is the number of nodes and `I`
is the number of intersections.  Since `N ~ V * log V` and `I ~ V`, constructing
the RTree and using it for the check takes `O(V * log V)` time, which is faster
than the brute-force `O(N^2)`.  This means it may be effectively free to
construct, if one is checking validity as well.

Polygon Validity
----------------

Any RTree can be used for quick and efficient polygon validity test (see note).
Since a valid polygon requires valid LinearRings for its shell and holes, assume
each ring has a prepared RTree.  Then the polygon validity check involves:
1. Envelope checks: `O(R^2)` where `R` is the number of rings.  Note for large
values of `R`, this can be reduced by making an RTree of rings.
2. Intersection checks between rings: `O(log V1 * log V2 + I)`, where `V1`
and `V2` are the numbers of vertices of each ring. See algorithm below.
3. Checking intersection matrix for cycles: `O(R)`

Algorithm sketch for checking the intersections between two LineStrings:
```
def query_other_intersections(rtree1: RTree, rtree2: RTree):
    stack = []
    stack.push(rtree1.root, rtree2.root)
    while not stack.is_empty():
        left, right = stack.pop()
        if not left.intersects(right):
            continue
        if left.is_leaf() and right.is_leaf():
            yield left, right
            continue
        if left.level >= right.level:
            for left_child in left.children:
                stack.push(left_child, right)
        else:
            for right_child in right.children:
                stack.push(left, right_child)
```

As above, for each pair of leaf nodes produced, the actual intersection check
needs to be performed.

Overlay Operations
------------------
I suspect Segment RTrees can also greatly speed up overlay operations
(intersection, different, symmentric difference, union). The intuition behind
this is that current algorithms, like the Vatti Polygon Clipping algorithm,
go through an check each segment from each Polygon. However, as humans we can
notice that segments far from the other geometry can be handled en-masse:
dropped for intersection, included for union, etc. The Segment RTree allows
us to check large sections of the shell, either dropping them (`O(1)`), or
(if packed coordinates) memcopying the range of them to the output. Memcopy
is technically `O(N)`, but it has such a low constant that it's practically
free for most algorithms.

For more details, see the dedicated note.




<!-- Links -->

[winding-number-test]: http://geomalgorithms.com/a03-_inclusion.html