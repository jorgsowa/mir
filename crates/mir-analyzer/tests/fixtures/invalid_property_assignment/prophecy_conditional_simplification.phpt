===description===
conditional type with identical branches collapses to constant type
===file===
<?php
/** @template T */
class Container {}

class TestFactory {
    /**
     * @return ($x is null ? Container<object> : Container<object>)
     */
    public function makeContainer($x): Container { return new Container(); }

    public Container $container;
}

$factory = new TestFactory();
$container = $factory->makeContainer(null);
$factory->container = $container;
===expect===
UnusedParam@9:35: Parameter $x is never used
