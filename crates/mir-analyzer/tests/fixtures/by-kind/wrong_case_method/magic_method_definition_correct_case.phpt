===description===
Correctly-cased magic method definitions are not reported.
===file===
<?php
class Bar {
    public function __construct() {}
    public function __destruct() {}
    public function __toString(): string { return "x"; }
    public function __callStatic(string $name, array $args): mixed { return null; }
    public function __debugInfo(): array { return []; }
    public function __invoke(): void {}
    public function __clone(): void {}
    public function __sleep(): array { return []; }
    public function __wakeup(): void {}
}
===expect===
