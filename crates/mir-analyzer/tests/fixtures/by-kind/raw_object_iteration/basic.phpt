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
}
===expect===
RawObjectIteration@9:15-9:17: Cannot iterate over non-iterable object 'Config'
