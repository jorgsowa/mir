===description===
RawObjectIteration does NOT fire for IteratorAggregate (extends Traversable)
===file===
<?php
class MyCollection implements \IteratorAggregate {
    private array $items;
    
    public function __construct(array $items) {
        $this->items = $items;
    }
    
    public function getIterator(): \ArrayIterator {
        return new \ArrayIterator($this->items);
    }
}

function process(MyCollection $col): void {
    yield from $col;
}
===expect===
