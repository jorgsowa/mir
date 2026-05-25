===description===
parameterized property also accepts bare actual (symmetric: bare is wildcard in both directions)
===file===
<?php
/** @template T */
class Box {}

class Container {
    public Box $typed;
}

$c = new Container();
// Bare Box (wildcard) should be accepted for Box-typed property
$bare = new Box();
$c->typed = $bare;
===expect===
