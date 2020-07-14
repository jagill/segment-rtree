# Rings and Polygon validity

An OGC Polygon consists of one ring as an exterior _shell_, and 0-to-many
rings as interior _holes_. Here, we define a _ring_ as a non-empty simple and
valid LineString -- represented by a sequence of coordinates -- with the
first and last coordinate equal. The non-empty, valid and simple conditions
on a LineString imply:

1. All coordinate values are finite.
2. There are at least 2 coordinates.
3. No two segments may intersect each other, _except_:
    1. Adjacent segments intersect at their common endpoint.
    2. The last segment may end at the beginning of the first segment.

We'll restrict this discussion to the 2-dimensional Euclidean plane. Here,
every ring divides the plane into an _exterior_, _interior_, and _boundary_.

For a Polygon to be valid, it must have the following properties:

1. All rings are non-empty, valid, simple, and closed.
2. Any two rings can intersect at at most one point.
3. Each hole ring must be contained in the shell ring.
4. No hole ring can be contained in another hole ring.
5. The interior of the Polygon must be a single connected component.

Any geometry has an _Envelope_, which is the minimal axis-aligned rectangle
that contains the geometry. It has attributes `{x_min, y_min, x_max, y_max}`,
which are the minimum/maximum of the `x` and `y` coordinates[^compact].

We'll discuss checks that ensure Polygon validity.

[^compact]: Note that the geometries are compact so we don't need infinum/supremum.

## Rings: A Prologue

Let's define a _coordinate_ as an x-y tuple of real numbers. In an algorithm,
this is best approximated as finite 64-bit floats. A _LineString_ can be
defined by a finite sequence of coordinates. LineStrings validity conditions 1.
and 2. can be checked easily on ingestion. For 3., a straightforward `O(V^2)`
(`V` is the number of vertices) nested loop can be used. If an RTree is built
on the segments (which takes `O(log V)` time), the check can be done in average
`O(log V + numIntersections)` time. Note that for a LineString, each segment
intersects its adjoining segments, so `numIntersections >= V`.

We quickly note that the LineString conditions also imply:

1. A LineString has at least 4 vertices.
2. A LineString's interior is a non-empty, open 2-dimensional subset of the
   plane.

Since we are addressing Polygon validity, we will think of rings as the
shells of hole-less polygons, which the corresponding notions of
containment/etc.  Given two rings `A` and `B` whose boundaries don't
intersect, we have one of three posibilities:

1. **Separate**: Their interiors are disjoint.
2. **A contains B**: `B`'s interior is a strict subset of `A`'s interior.
3. **B contains A**: `A`'s interior is a strict subset of `B`'s interior.

These possibilities also hold if the boundaries intersect at a single point.
We'll record this as a claim for future reference, but finish the proof below.

#### Claim 1: Rings whose boundaries intersect at 0 or 1 points have interiors that are disjoint, or one strictly contains the other.

To see this, note that if the interiors are equal then so are the boundaries.
We consider below the case of intersecions when the interior of neither `A`
nor `B` contains the other.

#### Claim 2: If two interiors intersect but neither is a subset of the other, the boundaries intersect at >=2 points.

**Proof**: Consider two rings, `A` and `B` (with boundaries `∂A` and `∂B`). If the boundaries intersect in an
infinite number of points, we are done, so consider the case that the
intersection of `∂A` and `∂B` is finite.

Since the interiors of `A` and `B` are open, their interiors' intersection `I`
is a finite collection of open connected components. Consider a single
connected component `I0` of `I`, and consider a point at its boundary `∂I0`.
As it's not in `I`, it must not be in both of the interiors of `A` and `B`,
If it is not in an interior of a ring, it must be in that ring's boundary.

If `∂I0` has every point from the boundary of a
single ring, say `A`, then `∂I0` forms a complete ring.  Since `A` is simple,
`∂I0 == ∂A`, and `A` is entirely contained in `B`, which is a contradiction.
Similarly, if `∂I0` has points either in `∂A` or `∂A ∩ ∂B`, the same argument
holds.

Thus, given our assumptions, `∂I0` must have points in both `∂A` and `∂B`.
For a each point only in `∂A` make a segment of all adjacent points also only
in `∂A` (and similarly for `∂B`). The points dividing these segments must be in 
`∂A ∩ ∂B`. Since we have at least one segment from `∂A` and one from `∂B`, there
must be at least two points in `∂A ∩ ∂B`, and we are done.

**TODO**: Since floats are actually discrete and finite, this claim should be
modified to take that into account.

#### Claim 3: A ring has a 2-dimensional envelope.

**Proof**: If an envelope had a degenerate dimension (say `y_max == y_min`),
the loop's segments must overlap: the 'return trip' must overlap the
'outgoing trip'.

#### Claim 4: If two rings have the same envelope, they intersect at >2 points.

**Proof**: Consider the `y_max` and `y_min` of the envelope. Then there are
points in `A` `p_max` and `p_min` that have the respective `y` values, and
similarly for `B` `q_max` and `q_min`. From above, `y_max != y_min` so `p_max
!= p_min` and `q_max != q_min`. If `p_max == q_max` and `p_min == q_min`, we
are done. Assume for a moment that neither of those equalities hold; we'll
see the single-equality case is the same.

Without loss of generality, assume `p_max.x < q_max.x`. Since `A` and `B` are
rings, there are two (continuous) paths from `q_max` to `q_min` and two from
`p_max` to `p_min`.

If `p_min.x > q_min.x`, then each of these paths must intersect each of the
corresponding paths from the other ring, which is at least 4 intersetions,
and we are done.

If `p_min.x < q_min.x`, consider the points `p_right` and `q_right` with `x`
coordintes `x_max`. The path between `p_max` and `p_right` must intersect the
path `q_max` to `q_min`, and this holds for `p_min` as well. Note that this
holds conversely. so there must be at least 4 intersection points.

To handle the "single-equality" case (say, `p_min == q_min`), the above
argument may only produce a single point of intersection in each direction.
But since the same argument holds for paths from `q_min` to `q_left`, we
still have two points of intersection.

**TODO**: Tighten/clarify argument.

Shells and Holes
----------------
A polygon consists of one shell ring, and some number of hole rings. A
polygon requires all its rings to be valid, so assume that has already been
checked. A polygon with no holes is valid. A polygon with a single hole is
valid if the hole is contained in the shell, and has at most one point of
intersection with the shell. We will call these conditions as the hole being
_shell-valid_. Polygons with multiple holes have that condition for each
hole, so we'll make a check for shell-validity.

#### Claim 5: If a shell's envelope does not strictly contain the hole's envelope, the hole is not shell-valid.

**Proof**: If a shell's envelope does not (weakly) contain a hole's envelope,
There most be at least one point of the hole (that is at the non-contained
envelope's boundary) that is not contained by the shell's envelope, and thus
not contained by the shell.

If a shell's envelope is equal to a hole's envelope, the boundaries of the
shell and hole intersect in at least 2 points (from the claim above), and
the hole is not shell-valid.

#### Claim 6: A hole is shell-valid if and only if these conditions hold:
1. The hole's envelope is strictly contained in the shell's envelope.
2. The hole's boundary intersects the shell's at 0 or 1 points.
3. A non-intersection point of the hole's boundary is contained in the shell.

**Proof**: Validity directly implies the three conditions.  Consider the
converse. Since there are only 0 or 1 intersetions, by Claim 1 the interior
of the hole is either disjoint from the interior of the shell, it strictly
contains the interior of the shell, or it is strictly contained by the
interior of the shell.

If the hole's interior strictly contains the shell's, the hole's envelope
would strictly contain the shell's, which is a contradiction. If a point
(excluding the intersection point) is contained in the shell, the interiors
cannot be disjoint. If the hole's interior is strictly contained in the
shell's, then the hole (including boundary) is contained within the shell
(including boundary). Thus the hole is shell-valid.

Holes and Holes
---------------
For a polygon to be valid, for any two holes:
1. Their interiors may not intersect, and
2. Their boundaries may intersect at only 0 or 1 points.

We say two holes are _pairwise-valid_ if these hold.  We can immediately see the
following claim.

#### Claim 7: Two holes whose envelopes don't intersect are pairwise-valid.

We only need to futher check validity for pairs of holes whose envelopes
intersect.  Those checks are very similar to the shell-hole validity condition.

#### Claim 8: Two holes are pairwise-valid if and only if these conditions hold:
1. Their boundaries intersect at 0 or 1 points.
2. A non-intersection point of each hole's boundary is *not* contained in the
other.

**Proof**: Validity implies these conditions directly. Consider the converse.
Since there are only 0 or 1 intersetions, by Claim 1 the interior of the hole
is either disjoint from the interior of the shell, it strictly contains the
interior of the shell, or it is strictly contained by the interior of the
shell.

If a (non-intersecting) point from hole `B`'s boundary is not contained
within `A`, `B`'s interior is not strictly contained within `A`, and
conversely. By condition 2, the two interiors must be disjoint and the holes
are pairwise-valid.

Connected interior
------------------
The final condition for polygon validity is that its interior -- defined as
the shell's interior minus the holes -- is a single connected component.

**Conjecture**: There are two ways this can be violated:

1. The holes can form a loop, with each hole intersecting it's neighbors at 1 point.
2. The holes can form a "bubble" on the shell, a loop that terminates on the shell.

I haven't figured out these checks, but here are my initial thoughts:
1. Each shell-hole and hole-hole check produces a possible intersection point.
2. This makes an intersection matrix for the various rings.
3. Define a cycle in this matrix as a sequence as a sequence rings, with
adjacent rings intersecting, such that each intersection has a different
coordinate.

**Conjecture**: The interior of the polygon is connected if and only if
there is no cycle in this matrix.

Test cases:

Valid?
```
-----------
|  /\     |
|  --  -- |
|      \/ |
-----------
```

Valid
```
-----------
|         |
|  |\/|   |
|  |/\|   |
|         |
-----------
```