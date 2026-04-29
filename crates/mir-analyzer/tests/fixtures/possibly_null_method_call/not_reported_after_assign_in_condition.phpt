===file===
<?php
class Foo {
    public function bar(): void {}
}
function f(): Foo|null { return null; }

// Regular = in if condition: $s must be narrowed to Foo after the falsy guard exits
function d(Foo|null $s): void {
    if (!($s = f())) { exit; }
    $s->bar();
}
===expect===
