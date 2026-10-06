===description===
Each call to a non-mutation-free method on $this inside a @psalm-immutable method
is reported individually, just like multiple property writes.
===file===
<?php

/** @psalm-immutable */
class Cache {
    private array $data = [];
    private int $hits = 0;

    public function refresh(): void {
        $this->clearData();
//      ^^^^^^^^^^^^^^^^^^ ImpureMethodCall: Calling impure method clearData() in a pure or immutable context
        $this->resetHits();
//      ^^^^^^^^^^^^^^^^^^ ImpureMethodCall: Calling impure method resetHits() in a pure or immutable context
    }

    private function clearData(): void {
        $this->data = [];
//      ^^^^^^^^^^^^^^^^ ImmutablePropertyModification: Assigning to property data of $this in an immutable context (@psalm-immutable class or @psalm-mutation-free method)
    }

    private function resetHits(): void {
        $this->hits = 0;
//      ^^^^^^^^^^^^^^^ ImmutablePropertyModification: Assigning to property hits of $this in an immutable context (@psalm-immutable class or @psalm-mutation-free method)
    }
}
