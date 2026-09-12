===description===
`instanceof` narrows literal array offsets.
===file===
<?php
class Foo { public function fooOnly(): void {} }
class Bar {}

/** @param array{item: Foo|Bar} $arr */
function test(array $arr): void {
    if ($arr['item'] instanceof Foo) {
        $arr['item']->fooOnly();
    }
}
===expect===
