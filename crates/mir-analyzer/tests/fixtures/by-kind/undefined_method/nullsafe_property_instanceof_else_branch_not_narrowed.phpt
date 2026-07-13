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
    }
}
===expect===
PossiblyNullPropertyFetch@15:8-15:17: Cannot access property $bar on possibly null value
PossiblyNullMethodCall@15:8-15:24: Cannot call method baz() on possibly null value
UndefinedMethod@15:8-15:24: Method Bar::baz() does not exist
