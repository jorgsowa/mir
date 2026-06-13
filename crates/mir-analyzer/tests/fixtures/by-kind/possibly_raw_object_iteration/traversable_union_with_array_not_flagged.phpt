===description===
PossiblyRawObjectIteration does NOT fire when yield-from is on an array or Traversable.
===file===
<?php
function items(array|\ArrayIterator $source): \Generator {
    yield from $source;
}
===expect===
