===description===
G11: `instanceof` type-guard narrowing never used to apply to array-offset
expressions — only plain variables and property accesses had a narrowing
arm. `$arr['item'] instanceof Foo` (a literal-keyed shape access) must
narrow the key's own stored type the same way `$this->item instanceof Foo`
already does, so a later read of the same key sees only `Foo`.
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
