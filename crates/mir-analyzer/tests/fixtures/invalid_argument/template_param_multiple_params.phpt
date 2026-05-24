===description===
multiple template parameters (non-empty union case) should not cause InvalidArgument
===file===
<?php
/** @template L @template R */
class Pair { }

class User { }
class Error { }

/**
 * @template L
 * @template R
 * @param Pair<L, R> $pair
 */
function processPair(Pair $pair): void {}

function test(): void {
    // Multiple templates - should not report InvalidArgument for L, R
    /** @var Pair<User, Error> $pair */
    $pair = new Pair();
    processPair($pair);
}
===expect===
UnusedParam@13:22: Parameter $pair is never used
