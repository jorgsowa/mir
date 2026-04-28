===file===
<?php
/**
 * @template T
 */
interface Container {
    /** @return T */
    public function get(): mixed;
}
interface Lockable {
    public function lock(): void;
}
/**
 * @template T
 */
class Wrapper {
    /**
     * @return Container<T>&Lockable
     */
    public function unwrap(): mixed { throw new \RuntimeException(); }
}
class Item { public function process(): void {} }
function test(): void {
    /** @var Wrapper<Item> $w */
    $w = new Wrapper();
    $w->unwrap()->get()->process();
}
===expect===

