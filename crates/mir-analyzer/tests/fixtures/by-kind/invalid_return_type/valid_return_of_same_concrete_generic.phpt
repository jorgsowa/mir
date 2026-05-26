===description===
returning the same concrete generic instantiation should not error
===file===
<?php
/** @template T */
class Box {}
/** @return Box<string> */
function makeBox(): mixed {
    /** @var Box<string> $b */
    $b = new Box();
    return $b;
}
===expect===
