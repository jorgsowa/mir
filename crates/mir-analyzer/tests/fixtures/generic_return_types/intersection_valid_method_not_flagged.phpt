===file===
<?php
/**
 * @template T
 */
interface Container {
    /** @return T */
    public function get(): mixed;
}
interface Taggable {
    public function tag(): string;
}
class Item { public function process(): void {} }
/**
 * @param Container<Item>&Taggable $c
 */
function test(object $c): void {
    /** @var Container<Item>&Taggable $c */
    $c->get()->process();
}
===expect===

