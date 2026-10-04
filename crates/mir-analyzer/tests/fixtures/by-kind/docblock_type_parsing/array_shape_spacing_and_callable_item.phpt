===description===
A space before `{` in an array shape, and a callable item whose return type has a colon, parse as shape types.
===file===
<?php
namespace Lib;

final class Shapes {
    /** @param array { a: int } $x */
    public function spaced(array $x): void {
        /** @mir-check $x is array{a: int} */
        $x;
    }

    /** @param array{callable(string): object, n: int} $x */
    public function positional(array $x): void {
        /** @mir-check $x is array{0: callable(string): object, n: int} */
        $x;
    }

    /** @param array{cb: callable(string): object, n: int} $x */
    public function keyed(array $x): void {
        /** @mir-check $x is array{cb: callable(string): object, n: int} */
        $x;
    }
}
===expect===
