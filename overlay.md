Segment RTrees and Overlay Operations
=====================================

**Work in Progress**
Overlay operations for geometries include:
1. Intersection: `A ∩ B`
2. Union: `A ∪ B`
3. Difference: `A \ B`
4. Symmetric Difference `A \ B ∪ B \ A`

I believe that Segment RTrees may be able to significantly speed up
overlay operations on geometries, by allowing large stretches of the geometries
to be handled in one operation.

First we introduce a simple datastructure called a `SegmentUnion`.

SegmentUnion
------------
An operation we will need to do repeated is storing subsections of LineStrings
to be handled later.  However, we want concatenate sections when possible:
if we first store the section `(3, 8)` then later `(9, 11)` then later `(8, 9)`,
that should be combined into a single segment `(3, 11)`.  We use a OrderedSet
as the base datastructure.  When we add a segment, for both `low` and `high`,
we check if the value is in the set.  If it is, we remove it.  If not, we add it.
To retrieve our stored segments, iterate through the OrderedSet, and each pair
(there will always be an even number of elements) will represent the `low`
and `high` of a maximally-concatenated segment.  Consider our example above:

```
segments = new SegmentUnion()  // set = {}
segments.add(3, 8)  // set = {3, 8}
segments.add(9, 11)  // set = {3, 8, 9, 11}
segments.add(8, 9)  // set = {3, 11}
for low, high in segments.get_all():
    print(low, high)
// Output: 3, 11
```

To define it:
```
SegmentUnion:
    set: OrderedSet[int]
    add: function(low: int, high: int):
        _add(low)
        _add(high)
    _add: function(entry: int):
        if entry in set:
            set.remove(entry)
        else:
            set.add(entry)
    get_all: function() -> List[(int, int)]:
        return set.iter().chunk(2).to_list()
    peek(): function() -> int:
        return set.first()
    is_empty(): function() -> bool:
        return set.is_empty()
    pop(): function() -> (int, int):
        assert set.length() >= 2
        return (set.remove_first(), set.remove_first())
```

Clipping LineStrings
--------------------
To clip a linestring to a rectangle, we first recurse down the SegRTree.  If a node
is disjoint from the rectangle, discard it.  If a node is contained within
the rectangle, store the `low` and `high` indices of that section in the
SegmentUnion.  If a node intersects but isn't contained by the rectangle,
recurse into its children, and if it's a leaf, store the `low` and `high` in
a MinHeap of intersecting segments.

To construct the output, conceptually we will traverse the segments (in order)
that are either contained within or intersect the rectangle.  If they are
contained, we will copy the range into the output.  If the segment intersects
the rectangle, we will determine the intersection and add it to the output.
In both cases we will try to attach the result to the previous, if possible.

For example, consider the case with a contained section (3, 11), and intersecting segments
(2, 3) and (10, 11).  Let's say the intersection of the rectangle with (2, 3)
is `(p, coords[3])` and that with (10, 11) is `(coords[10], q)`.  Then the 
constructed linestring would be `(p, coords[3..1], q)`.  That would complete
that linestring, and a new linestring would be constructed 

The algorithm is roughly as follows:

```
def clip_linestring(
    rect: Rectangle, coordinates: Array[Coordinate], rtree: SegRTree
) -> List[LineString]:
    // First, find the segments that are contained or intersecting the rect
    contained = SegmentUnion()
    intersects = MinHeap[(int, int)]()
    stack = []
    stack.push(rtree.root)
    while not stack.is_empty():
        node = stack.pop()
        if not rect.intersects(node):
            continue
        else if rect.contains(node):
            segments.add(node.low, node.high)
        else if node.is_leaf():
            intersects.push(node.low, node,high)
        else:
            for child in node.children():
                stack.push(child)

    // Now construct the intersection from our elements
    results = List[List[Coordinate]]()
    capacity = ... // Something large enough; find a way to calculate
    out_coords = Coordinate[capacity]
    out_index = 0
    last_index = -1

    def flush_output():
        if not out_coords.is_empty():
            results.push(out_coords.clone())
            out_coords.clear()
            out_index = 0

    def pop_contained():
        let low, high = contained.pop()
        if low == last_index:
            low += 1
        else:
            flush_output()
        copy(out_coords, coordinates, out_index, delta)
        last_index = high
        out_index += high - low + 1

    def pop_intersects():
        let low, high = intersects.pop()
        let seg_start, set_end = coordinates[low], coordinates[high]
        let seg = Segment{seg_start, seg_end}
        if not rect.intersects(seg):
            return
        let isxn_start, isxn_end = rect.intersection(seg)
        if low != last_index:
            flush_output()
            out_coords.push(isxn_start)
            out_index += 1
        if isxn_end != isxn_start:
            out_coors.push(isxn_end)
            out_index += 1
        if isxn_end == seg_end:
            last_index = high

    while not (contained.is_empty() or intersects.is_empty()):
        // One of the two will have the lowest/next index
        if segments.peek() < intersections.peek()[0]:
            pop_contained()
        else:
            pop_intersects()

    while not contained.is_empty():
        pop_contained()
    while not intersects.is_empty():
        pop_intersects()
    flush_output
    
    return results
```
        
        
