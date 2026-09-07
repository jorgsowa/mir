===description===
`is_a($obj->prop, X::class)` must narrow the property receiver like the
already-correct `$obj->prop instanceof X` sibling.
===config===
suppress=MissingConstructor,PossiblyNullArgument
===file===
<?php
class Base {}
class Foo extends Base {
    public function fooMethod(): void {}
}
class Container {
    public ?Base $item;
}
function f(Container $c): void {
    if (is_a($c->item, Foo::class)) {
        $c->item->fooMethod();
    }
}
===expect===
