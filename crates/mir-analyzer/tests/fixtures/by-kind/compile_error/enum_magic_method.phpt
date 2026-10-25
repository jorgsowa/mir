===description===
PHP rejects most magic methods on an enum, also when a used trait brings one in; `__call`, `__callStatic` and `__invoke` are allowed, and `__debugInfo` only from PHP 8.6.
===config===
<mir>
  <phpVersion>8.5</phpVersion>
</mir>
===file===
<?php
trait Printable {
    public function __toString(): string { return ''; }
}

enum Suit {
//<^^^^^^^^^^^ CompileError: Compile error: Enum Suit cannot include magic method __toString [enum-magic-method]
    use Printable;
    case Hearts;

    public function __construct() {}
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ CompileError: Compile error: Enum Suit cannot include magic method __construct [enum-magic-method]
    public function __get(string $name): int { return 1; }
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ CompileError: Compile error: Enum Suit cannot include magic method __get [enum-magic-method]
    public function __debugInfo(): array { return []; }
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ CompileError: Compile error: Enum Suit cannot include magic method __debugInfo [enum-magic-method]
    public function __call(string $name, array $args): int { return 1; }
    public static function __callStatic(string $name, array $args): int { return 1; }
    public function __invoke(): int { return 1; }
}
