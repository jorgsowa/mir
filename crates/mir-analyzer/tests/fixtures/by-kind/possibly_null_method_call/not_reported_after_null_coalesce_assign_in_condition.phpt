===description===
not reported after null coalesce assign in condition
===file===
<?php
class Foo {
    public function bar(): void {}
}
function f(): Foo|null { return null; }

// ??= in if condition: $s must be narrowed to Foo (non-null) after the falsy guard exits
function a(Foo|null $s): void {
    if (!($s ??= f())) { exit; }
    $s->bar();
}
===expect===
===ignore===
TODO
