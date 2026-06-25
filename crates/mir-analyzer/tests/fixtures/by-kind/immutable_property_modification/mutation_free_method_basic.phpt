===description===
@psalm-mutation-free on a method prevents $this->prop assignment, even when the
class itself is not @psalm-immutable.
===file===
<?php

class Counter {
    public int $count = 0;

    /** @psalm-mutation-free */
    public function reset(): void {
        $this->count = 0;
    }

    public function increment(): void {
        $this->count++;
    }
}
===expect===
ImmutablePropertyModification@8:8-8:24: Assigning to property count of $this in an immutable context (@psalm-immutable class or @psalm-mutation-free method)
