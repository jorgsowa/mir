===description===
keyed array property type resolved
===file===
<?php
/**
 * @template T
 */
class Box {
    /** @return array{item: T} */
    public function wrap(): array { return []; }
}
class Item { public function process(): void {} }
function test(): void {
    /** @var Box<Item> $box */
    $box = new Box();
    $result = $box->wrap();
    $result['item']->undefinedMethod();
}
===expect===
UndefinedMethod@14:4-14:38: Method Item::undefinedMethod() does not exist
