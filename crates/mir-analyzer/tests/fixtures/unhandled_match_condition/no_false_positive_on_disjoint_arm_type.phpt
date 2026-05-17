===description===
no false positive when match arm condition type is disjoint from subject type
===file===
<?php
class Foo {
    public function fooMethod(): void {}
}
class Bar {
    public function barMethod(): void {}
}

/** @param Foo $x */
function test(Foo $x): void {
    match ($x) {
        new Bar() => $x->fooMethod(),
    };
}
===expect===

