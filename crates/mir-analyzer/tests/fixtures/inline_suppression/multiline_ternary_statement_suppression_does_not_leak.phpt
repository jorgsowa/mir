===description===
A suppression above a multi-line ternary statement does not cover the next statement or other issue kinds.
===file===
<?php
function takeInt(int $x): void { echo $x; }

function f(bool $c): void {
    /** @psalm-suppress InvalidArgument */
    $c
        ? takeInt('a')
        : undefined_fn();
//        ^^^^^^^^^^^^^^ UndefinedFunction: Function undefined_fn() is not defined

    takeInt('b');
//          ^^^ InvalidArgument: Argument $x of takeInt() expects 'int', got '"b"'
}
