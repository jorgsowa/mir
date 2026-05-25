===description===
conditional type with identical branches collapses to constant type
===file===
<?php
// Methods that return (cond is T ? T : T) should resolve to T
class TestFactory {
    /**
     * @return ($x is null ? Container<object> : Container<object>)
     */
    public function makeContainer($x): Container {}

    /** @var Container */
    public $container;
}

class Container<T> {}

$factory = new TestFactory();
$container = $factory->makeContainer(null);
$factory->container = $container;
===expect===
