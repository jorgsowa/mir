===description===
Detect missing template implements native
===file===
<?php
final class C implements Iterator {
    public function current(): mixed {
        return 0;
    }
    public function key(): mixed {
        return 0;
    }
    public function next(): void {
    }
    public function rewind(): void {
    }
    public function valid(): bool {
        return false;
    }
}

===expect===
