===description===
`instanceof` narrows nested array offsets.
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
