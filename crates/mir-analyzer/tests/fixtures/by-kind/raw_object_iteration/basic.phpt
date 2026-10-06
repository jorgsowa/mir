===description===
RawObjectIteration fires when yield-from is used with a non-Traversable object.
===file===
<?php
class Config {
    public string $host = "localhost";
    public int $port = 8080;
}

function items(): \Generator {
    $c = new Config();
    yield from $c;
//             ^^ RawObjectIteration: Cannot iterate over non-iterable object 'Config'
}
