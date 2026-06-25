===description===
PossiblyRawObjectIteration does NOT fire when all types in a union implement Traversable.
===file===
<?php
function gen(\ArrayIterator|\Generator $source): \Generator {
    yield from $source;
}
===expect===
