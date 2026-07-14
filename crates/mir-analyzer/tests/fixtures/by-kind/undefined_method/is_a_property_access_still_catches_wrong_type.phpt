===description===
Negative counterpart: `is_a($obj->prop, X::class)` narrowing must not
over-widen — a method that only exists on an unrelated sibling class is
still flagged.
===config===
suppress=MissingConstructor,PossiblyNullArgument
===file===
<?php
class Base {}
class Foo extends Base {}
class Bar extends Base {
    public function barMethod(): void {}
}
class Container {
    public ?Base $item;
}
function f(Container $c): void {
    if (is_a($c->item, Foo::class)) {
        $c->item->barMethod();
    }
}
===expect===
UndefinedMethod@12:8-12:29: Method Foo::barMethod() does not exist
