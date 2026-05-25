===description===
bare generic accepts parameterized form with same FQCN
===file===
<?php
class Container {
    /** @var GenericWrapper */
    public $bare;

    /** @var GenericWrapper<string> */
    public $typed;
}

class GenericWrapper<T> {}

$c = new Container();
$wrapper = new GenericWrapper();
$c->bare = $wrapper;
$c->typed = $wrapper;
===expect===
