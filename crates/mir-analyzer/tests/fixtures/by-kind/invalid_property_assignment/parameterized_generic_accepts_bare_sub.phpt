===description===
parameterized value is accepted by same-FQCN bare PHP-typed property (wildcard suppresses param mismatch)
===file===
<?php
/** @template T */
class Box {}

class BoxFactory {
    /** @return Box<string> */
    public function makeString(): Box { return new Box(); }

    /** @return Box<int> */
    public function makeInt(): Box { return new Box(); }
}

class Container {
    public Box $box;
}

$factory = new BoxFactory();
$c = new Container();

$stringBox = $factory->makeString();
$intBox = $factory->makeInt();
/** @mir-check $stringBox is Box<string> */
/** @mir-check $intBox is Box<int> */
// Bare property accepts both parameterized values regardless of type param
$c->box = $stringBox;
$c->box = $intBox;
===expect===
