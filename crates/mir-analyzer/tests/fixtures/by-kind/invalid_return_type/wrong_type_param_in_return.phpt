===description===
returning a generic instantiation with wrong type param should error
===file===
<?php
/** @template T */
class Box {}
/** @return Box<string> */
function makeStringBox(): mixed {
    /** @var Box<int> $b */
    $b = new Box();
    return $b;
}
===expect===
InvalidReturnType@8:5-8:15: Return type 'Box<int>' is not compatible with declared 'Box<string>'
