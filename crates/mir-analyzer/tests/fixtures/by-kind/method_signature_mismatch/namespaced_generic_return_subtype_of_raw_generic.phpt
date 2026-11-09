===description===
Inside a namespace, a return of `ArrayIterator<string, ItemList<Item>>` satisfies a parent bound to `Traversable<string, ItemList>`; a global class isn't looked up under the namespace.
===file===
<?php
namespace App;
/** @template T */
class ItemList {}
class Item {}
/** @implements \IteratorAggregate<string, ItemList> */
abstract class Base implements \IteratorAggregate {
    /** @return \ArrayIterator<string, ItemList<Item>> */
    public function getIterator(): \ArrayIterator { return new \ArrayIterator([]); }
}
