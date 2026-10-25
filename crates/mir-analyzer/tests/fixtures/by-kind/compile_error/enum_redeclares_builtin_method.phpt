===description===
An enum may not declare `cases()`, nor a backed enum `from()` / `tryFrom()`; PHP's own method takes precedence over a trait's copy, which is not an error.
===file===
<?php
namespace App;

trait Lookup {
    public static function from(string $v): string { return $v; }
}

enum Pure {
    use Lookup;
    case A;

    public function cases(): array { return []; }
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ CompileError: Cannot redeclare App\Pure::cases() [duplicate-method]
    public static function tryFrom(int $v): ?self { return $v > 0 ? self::A : null; }
}

enum Backed: int {
    use Lookup;
    case A = 1;

    public static function TryFrom(int|string $v): ?static { return null; }
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ CompileError: Cannot redeclare App\Backed::tryfrom() [duplicate-method]
}
