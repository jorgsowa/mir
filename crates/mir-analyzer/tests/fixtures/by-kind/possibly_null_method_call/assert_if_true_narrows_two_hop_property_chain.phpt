===description===
apply_one_assertion's property-access arm only matched a bare 1-hop
receiver (`extract_any_prop_access`, which requires the object to itself
be a plain variable) — a 2-hop chain (`$c->box->inner`) silently no-oped
the whole assertion, unlike `@psalm-self-out`'s write-back, which already
supports the same synthetic 2-hop key via `extract_chained_prop_access`.
===config===
suppress=MissingConstructor,MissingPropertyType
===file===
<?php
class Foo {
    public function bar(): void {}
}
class Box {
    /** @var Foo|null */
    public $inner;
}
class Container {
    public Box $box;
}
/**
 * @param mixed $value
 * @psalm-assert-if-true Foo $value
 */
function isFoo($value): bool {
    return $value instanceof Foo;
}
function test(Container $c): void {
    if (isFoo($c->box->inner)) {
        $c->box->inner->bar();
    }
}
===expect===
