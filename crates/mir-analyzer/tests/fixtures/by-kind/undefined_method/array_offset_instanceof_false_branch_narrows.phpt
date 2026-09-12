===description===
Negated and early-return `instanceof` guards narrow array offsets correctly.
===file===
<?php
class Foo { public function fooOnly(): void {} }
class Bar {}

/** @param array{item: Foo|Bar} $arr */
function negatedGuard(array $arr): void {
    if (!($arr['item'] instanceof Foo)) {
        return;
    }
    $arr['item']->fooOnly();
}

/** @param array{item: Foo|Bar} $arr */
function earlyReturnOnMatch(array $arr): void {
    if ($arr['item'] instanceof Foo) {
        return;
    }
    $arr['item']->fooOnly();
}
===expect===
UndefinedMethod@18:4-18:27: Method Bar::fooOnly() does not exist
