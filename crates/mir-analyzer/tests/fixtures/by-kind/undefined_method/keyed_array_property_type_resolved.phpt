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
UndefinedMethod@14:5-14:39: Method Item::undefinedMethod() does not exist
