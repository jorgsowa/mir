===description===
A named `@var Foo|null $x` docblock above a multi-variable `static $x = null, $y = 0;`
declaration must apply only to the named `$x`, leaving `$y`'s type derived from its own
literal initializer untouched.
===config===
suppress=UnusedParam
===file===
<?php
class Foo {
    public function bar(): void {}
}

function test(): void {
    /** @var Foo|null $x */
    static $x = null, $y = 0;
    if ($x !== null) {
        $x->bar();
    }
    if ($y !== 0) {
        echo 'unreachable';
    }
}
===expect===
