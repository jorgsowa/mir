===description===
G11, nested path: `$arr['a']['b'] instanceof Foo` (two literal keys deep)
must narrow the innermost key, same as the single-level case.
===file===
<?php
class Foo { public function fooOnly(): void {} }
class Bar {}

/** @param array{a: array{b: Foo|Bar}} $arr */
function test(array $arr): void {
    if ($arr['a']['b'] instanceof Foo) {
        $arr['a']['b']->fooOnly();
    }
}
===expect===
