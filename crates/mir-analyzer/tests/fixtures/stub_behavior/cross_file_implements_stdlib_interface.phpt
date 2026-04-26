===file:Collection.php===
<?php
class NumberList implements \Iterator {
    private array $items;
    private int $pos = 0;

    public function __construct(array $items) { $this->items = $items; }
    public function current(): mixed { return $this->items[$this->pos]; }
    public function key(): int { return $this->pos; }
    public function next(): void { $this->pos++; }
    public function rewind(): void { $this->pos = 0; }
    public function valid(): bool { return isset($this->items[$this->pos]); }
}
===file:Main.php===
<?php
function process(\Iterator $it): void {
    foreach ($it as $v) {
        echo $v;
    }
}
$list = new NumberList([1, 2, 3]);
process($list);
===expect===
