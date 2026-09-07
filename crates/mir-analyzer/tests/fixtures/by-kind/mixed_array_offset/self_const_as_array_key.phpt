===description===
Bitmask class constants used as array keys should not emit MixedArrayOffset.
self::INT_CONST / static::INT_CONST return their literal type, not mixed.
===config===
suppress=UnusedVariable
===file===
<?php
class Cache {
    const READ  = 1;
    const WRITE = 2;
    const EXEC  = 4;

    /** @var array<int, string> */
    private array $store = [];

    public function set(string $value): void {
        $key = self::READ | self::WRITE;
        $this->store[$key] = $value;

        $this->store[self::EXEC] = $value;
        $this->store[static::READ] = $value;
    }
}
===expect===
