===description===
RawObjectIteration does NOT fire when yield-from is used on a Traversable object.
===file===
<?php
function items(\ArrayIterator $iter): \Generator {
    yield from $iter;
}
===expect===
