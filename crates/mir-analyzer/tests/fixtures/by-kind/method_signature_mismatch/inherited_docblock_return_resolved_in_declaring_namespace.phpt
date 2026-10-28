===description===
A namespaced class overriding `IteratorAggregate::getIterator()` is compared with the stub's `Traversable<TKey, TValue>`, not a `Traversable` in the child's namespace
===file===
<?php
namespace App;

/** @implements \IteratorAggregate<string, mixed> */
class Bag implements \IteratorAggregate {
    /** @var array<string, mixed> */
    private array $items = [];

    /** @return \ArrayIterator<string, mixed> */
    public function getIterator(): \ArrayIterator {
        return new \ArrayIterator($this->items);
    }
}

/** @implements \IteratorAggregate<string, int> */
class Counts implements \IteratorAggregate {
    /** @return \ArrayIterator<string, string> */
    public function getIterator(): \ArrayIterator {
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MethodSignatureMismatch: Method App\Counts::getiterator() signature mismatch: return type 'ArrayIterator<string, string>' is not a subtype of parent 'Traversable<string, int>|array<int|string, int>'
        return new \ArrayIterator(['a' => 'b']);
    }
}
