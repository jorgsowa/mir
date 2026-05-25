===description===
bare generic accepts parameterized form with multiple type params
===file===
<?php
class Config {
    /** @var Pair */
    public $bare;

    /** @var Pair<string, int> */
    public $typed;
}

class Pair<K, V> {}

$c = new Config();
$pair = new Pair();
$c->bare = $pair;
$c->typed = $pair;
===expect===
