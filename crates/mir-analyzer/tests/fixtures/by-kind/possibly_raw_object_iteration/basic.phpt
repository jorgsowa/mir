===description===
PossiblyRawObjectIteration fires when yield-from is used on a union that includes a non-Traversable object.
===file===
<?php
class Config {
    public string $host = "localhost";
}

function items(Config|\ArrayIterator $source): \Generator {
    yield from $source;
//             ^^^^^^^ PossiblyRawObjectIteration: Cannot iterate over possibly non-iterable object 'Config|ArrayIterator'
}
