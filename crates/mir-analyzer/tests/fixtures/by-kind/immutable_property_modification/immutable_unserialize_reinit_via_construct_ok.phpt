===description===
Re-initializing `$this` through `__construct()` inside the unserialization and
clone hooks of a @psalm-immutable class is not an impure method call.
===file===
<?php

/** @psalm-immutable */
final class Id {
    public function __construct(public string $value) {}

    public function unserialize(string $data): void {
        $this->__construct($data);
    }

    /** @param array{value: string} $data */
    public function __unserialize(array $data): void {
        $this->__construct($data['value']);
    }

    public function __wakeup(): void {
        $this->__construct($this->value);
    }

    public function __clone() {
        $this->__construct($this->value);
    }
}
