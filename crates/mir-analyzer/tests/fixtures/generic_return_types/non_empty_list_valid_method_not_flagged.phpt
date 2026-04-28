===file===
<?php
/**
 * @template T
 */
class Box {
    /** @return non-empty-list<T> */
    public function items(): array { return []; }
}
class Item { public function process(): void {} }
function test(): void {
    /** @var Box<Item> $box */
    $box = new Box();
    foreach ($box->items() as $item) {
        $item->process();
    }
}
===expect===
