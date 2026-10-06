===description===
@phpstan-mutation-free is recognized as an alias for @psalm-mutation-free.
===file===
<?php

class Cache {
    public array $data = [];

    /** @phpstan-mutation-free */
    public function clear(): void {
        $this->data = [];
//      ^^^^^^^^^^^^^^^^ ImmutablePropertyModification: Assigning to property data of $this in an immutable context (@psalm-immutable class or @psalm-mutation-free method)
    }
}
