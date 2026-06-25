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
        $this->resetHits();
    }

    private function clearData(): void {
        $this->data = [];
    }

    private function resetHits(): void {
        $this->hits = 0;
    }
}
===expect===
ImpureMethodCall@9:8-9:26: Calling impure method clearData() in a pure or immutable context
ImpureMethodCall@10:8-10:26: Calling impure method resetHits() in a pure or immutable context
ImmutablePropertyModification@14:8-14:24: Assigning to property data of $this in an immutable context (@psalm-immutable class or @psalm-mutation-free method)
ImmutablePropertyModification@18:8-18:23: Assigning to property hits of $this in an immutable context (@psalm-immutable class or @psalm-mutation-free method)
