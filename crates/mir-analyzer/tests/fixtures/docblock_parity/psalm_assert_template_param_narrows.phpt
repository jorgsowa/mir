===description===
psalm assert template param narrows
===file===
<?php
/**
 * @template T
 * @param T|null $value
 * @psalm-assert T $value
 */
function assert_not_null($value): void {
    if ($value === null) { throw new \RuntimeException(); }
}

class Bar { public function ping(): void {} }

function test(?Bar $x): void {
    assert_not_null($x);
    $x->ping();
    $x->missing();
}
===expect===
UndefinedMethod@16:5: Method Bar::missing() does not exist
