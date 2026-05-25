===description===
bare generic accepts parameterized form with multiple type params
===file===
<?php
/**
 * @template K
 * @template V
 */
class Pair {}

class Config {
    public Pair $bare;
}

$c = new Config();
$pair = new Pair();
$c->bare = $pair;
===expect===
