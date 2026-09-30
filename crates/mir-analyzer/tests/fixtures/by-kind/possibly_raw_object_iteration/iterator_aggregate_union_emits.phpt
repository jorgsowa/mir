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
//             ^^^^^^^ PossiblyRawObjectIteration: Cannot iterate over possibly non-iterable object 'Stream|Items'
}
===expect===
