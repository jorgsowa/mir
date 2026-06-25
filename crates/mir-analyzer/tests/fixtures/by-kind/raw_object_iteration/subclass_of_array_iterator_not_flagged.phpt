===description===
RawObjectIteration does NOT fire for a class that extends ArrayIterator.
===file===
<?php
class TypedList extends \ArrayIterator {}

function gen(TypedList $list): \Generator {
    yield from $list;
}
===expect===
