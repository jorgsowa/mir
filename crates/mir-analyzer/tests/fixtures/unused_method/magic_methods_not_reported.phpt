===config===
find_dead_code=true
===file===
<?php
class Magic {
    private function __construct() {}
    private function __destruct() {}
    private function __call(string $name, array $arguments): mixed { return null; }
    private static function __callStatic(string $name, array $arguments): mixed { return null; }
    private function __get(string $name): mixed { return null; }
    private function __set(string $name, mixed $value): void {}
    private function __isset(string $name): bool { return false; }
    private function __unset(string $name): void {}
    private function __sleep(): array { return []; }
    private function __wakeup(): void {}
    private function __serialize(): array { return []; }
    private function __unserialize(array $data): void {}
    private function __toString(): string { return ''; }
    private function __invoke(): void {}
    private function __clone(): void {}
    private function __debugInfo(): array { return []; }
}
===expect===
