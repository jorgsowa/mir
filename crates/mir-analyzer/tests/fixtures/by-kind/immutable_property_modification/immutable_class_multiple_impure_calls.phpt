===description===
Each mutation inside helpers of a @psalm-immutable class is reported individually
where it happens; the `$this->helper()` calls themselves are not flagged.
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
//      ^^^^^^^^^^^^^^^^ ImmutablePropertyModification: Assigning to property data of $this in an immutable context (@psalm-immutable class or @psalm-mutation-free method)
    }

    private function resetHits(): void {
        $this->hits = 0;
//      ^^^^^^^^^^^^^^^ ImmutablePropertyModification: Assigning to property hits of $this in an immutable context (@psalm-immutable class or @psalm-mutation-free method)
    }
}
