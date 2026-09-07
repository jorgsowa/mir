===description===
`static $x = null;` clobbered a preceding `@var Foo|null $x` docblock annotation:
the static var's type was computed purely from the literal initializer (`null` ->
`TNull`), overwriting the wider annotated type, so a later `$x !== null` check was
wrongly reported as always false. The post-narrow `@var` reapplication that already
ran for plain `$x = expr;` assignments now also runs for `static` declarations.
===config===
suppress=UnusedParam
===file===
<?php
class Foo {
    public function bar(): void {}
}

function test(): void {
    /** @var Foo|null $x */
    static $x = null;
    if ($x !== null) {
        $x->bar();
    }
}
===expect===
