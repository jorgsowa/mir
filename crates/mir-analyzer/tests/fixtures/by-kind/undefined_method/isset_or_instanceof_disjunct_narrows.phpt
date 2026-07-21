===description===
`!isset($x) || $x instanceof Foo` narrows $x to Foo|null (excluding Bar)
===file===
<?php
interface Fooable { public function fooOnly(): void; }
class Foo implements Fooable { public function fooOnly(): void {} }
class Bar {}

/** @param Foo|Bar|null $x */
function narrows($x): void {
    if (!isset($x) || $x instanceof Foo) {
        if ($x !== null) {
            $x->fooOnly();
        }
    }
}

/** @param Foo|Bar|null $x */
function still_flags_undefined_method($x): void {
    if (!isset($x) || $x instanceof Foo) {
        if ($x !== null) {
            $x->notAMethod();
        }
    }
}
===expect===
UndefinedMethod@19:12-19:28: Method Foo::notAMethod() does not exist
