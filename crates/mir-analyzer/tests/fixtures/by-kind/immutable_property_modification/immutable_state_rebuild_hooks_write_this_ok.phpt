===description===
`unserialize()`, `__unserialize()` and `__clone()` rebuild the state of a
@psalm-immutable object, so they may write `$this` properties; `__wakeup()`
and ordinary methods may not.
===file===
<?php

/** @psalm-immutable */
final class Id {
    public string $value;
    public int $count = 0;

    public function __construct(string $value) {
        $this->value = $value;
    }

    public function unserialize(string $data): void {
        $this->value = $data;
    }

    /** @param array{value: string} $data */
    public function __unserialize(array $data): void {
        $this->value = $data['value'];
    }

    public function __clone() {
        $this->count = 0;
    }

    public function __wakeup(): void {
        $this->count = 1;
//      ^^^^^^^^^^^^^^^^ ImmutablePropertyModification: Assigning to property count of $this in an immutable context (@psalm-immutable class or @psalm-mutation-free method)
    }

    public function bump(): void {
        $this->count = 2;
//      ^^^^^^^^^^^^^^^^ ImmutablePropertyModification: Assigning to property count of $this in an immutable context (@psalm-immutable class or @psalm-mutation-free method)
    }
}
