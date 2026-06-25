===description===
PossiblyRawObjectIteration fires for a union of IteratorAggregate and non-Traversable.
===file===
<?php
class Stream {}

class Items implements \IteratorAggregate {
    public function getIterator(): \ArrayIterator {
        return new \ArrayIterator([]);
    }
}

function gen(Stream|Items $source): \Generator {
    yield from $source;
}
===expect===
PossiblyRawObjectIteration@11:15-11:22: Cannot iterate over possibly non-iterable object 'Stream|Items'
