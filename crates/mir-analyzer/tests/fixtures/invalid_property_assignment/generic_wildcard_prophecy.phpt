===description===
bare generic accepts parameterized form with same FQCN
===file===
<?php
/** @template T */
class GenericWrapper {}

class Container {
    public GenericWrapper $bare;

    /** @var GenericWrapper<string> */
    public GenericWrapper $typed;
}

$c = new Container();
$wrapper = new GenericWrapper();
$c->bare = $wrapper;
$c->typed = $wrapper;
===expect===
