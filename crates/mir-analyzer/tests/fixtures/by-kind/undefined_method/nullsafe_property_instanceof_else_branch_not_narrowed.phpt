===description===
`$obj?->prop instanceof ClassName` narrowing must not leak into the else branch
===file===
<?php

class Bar {}
class Baz extends Bar {
    public function baz(): void {}
}
class Foo {
    public ?Bar $bar = null;
}

function f(?Foo $foo): void {
    if ($foo?->bar instanceof Baz) {
        echo 'ok';
    } else {
        $foo->bar->baz();
//      ^^^^^^^^^^^^^^^^ PossiblyNullMethodCall: Cannot call method baz() on possibly null value
//      ^^^^^^^^^ PossiblyNullPropertyFetch: Cannot access property $bar on possibly null value
//      ^^^^^^^^^^^^^^^^ UndefinedMethod: Method Bar::baz() does not exist
    }
}
