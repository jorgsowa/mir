===description===
bare PHP-typed property accepts parameterized values of same class regardless of type param
===file===
<?php
/** @template T */
class Container {}

class ContainerFactory {
    /** @return Container<string> */
    public function makeString(): Container { return new Container(); }

    /** @return Container<int> */
    public function makeInt(): Container { return new Container(); }
}

class Config {
    public Container $prop;
}

$factory = new ContainerFactory();
$c = new Config();

$s = $factory->makeString();
$i = $factory->makeInt();
/** @mir-check $s is Container<string> */
/** @mir-check $i is Container<int> */
// Both parameterized values accepted by bare property
$c->prop = $s;
$c->prop = $i;
===expect===
MissingConstructor@13:0-13:14: Class Config has uninitialized properties but no constructor
