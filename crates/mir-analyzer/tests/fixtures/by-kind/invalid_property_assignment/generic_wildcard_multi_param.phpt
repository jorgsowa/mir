===description===
bare PHP-typed property accepts parameterized value for class with multiple type params
===file===
<?php
/**
 * @template K
 * @template V
 */
class Pair {}

class PairFactory {
    /**
     * @template K
     * @template V
     * @return Pair<K, V>
     */
    public function make(): Pair { return new Pair(); }
}

class Config {
    public Pair $bare;
}

$factory = new PairFactory();
$c = new Config();
$pair = $factory->make();
/** @mir-check $pair is Pair<mixed, mixed> */
$c->bare = $pair;
===expect===
MissingConstructor@17:0-17:14: Class Config has uninitialized properties but no constructor
