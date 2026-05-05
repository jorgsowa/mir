===description===
returning Box<Box<int>> when Box<Box<string>> is declared should error
===file===
<?php
/** @template T */
class Box {}
/** @return Box<Box<string>> */
function makeBox(): mixed {
    /** @var Box<Box<int>> $b */
    $b = new Box();
    return $b;
}
===expect===
InvalidReturnType@8:4: Return type 'Box<Box<int>>' is not compatible with declared 'Box<Box<string>>'
