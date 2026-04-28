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
UndefinedMethod: Method Item::undefinedMethod() does not exist
