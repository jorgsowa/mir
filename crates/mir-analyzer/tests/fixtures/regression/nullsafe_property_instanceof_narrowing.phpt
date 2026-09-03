===description===
`$obj?->prop instanceof ClassName` must narrow the property's type in the
guarded branch, the same way `$obj->prop instanceof ClassName` already does —
narrowing.rs's Instanceof arm only checked extract_prop_access (plain `->`),
never extract_nullsafe_prop_access, so the guard had no effect and the
property stayed at its declared (nullable base-class) type.
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
        $foo->bar->baz();
    }
}
===expect===
