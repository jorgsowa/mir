===description===
magic methods not reported
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
  <issueHandlers>
    <NonPublicMagicMethod errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Magic {
//<^ +17:1 UnusedClass: Class Magic is never referenced
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
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ OverriddenMethodAccess: Access level to Magic::__toString() must be public (as in class Stringable)
    private function __invoke(): void {}
    private function __clone(): void {}
    private function __debugInfo(): array { return []; }
}
