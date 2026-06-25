===description===
A sibling method without @psalm-mutation-free is still allowed to write
$this properties.
===file===
<?php

class Repository {
    public array $items = [];

    /** @psalm-mutation-free */
    public function count(): int {
        return count($this->items);
    }

    public function add(mixed $item): void {
        $this->items[] = $item;
    }
}
===expect===
