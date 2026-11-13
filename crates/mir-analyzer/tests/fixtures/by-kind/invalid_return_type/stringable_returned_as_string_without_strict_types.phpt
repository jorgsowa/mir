===description===
Without `strict_types`, PHP casts a Stringable object returned for `string` by
calling `__toString()`, as it does for arguments. A non-Stringable object and
any object under `strict_types` are still `InvalidReturnType`.
===file:Weak.php===
<?php
final class Line implements Stringable {
    public function __toString(): string { return 'l'; }
}

final class Renderer {
    public function __toString(): string { return 'r'; }

    public function render(): string { return $this; }

    public function line(): string { return new Line(); }

    public function maybe(): ?string { return rand(0, 1) ? $this : null; }

    public function arrow(): Closure { return fn (): string => new Line(); }
}

final class Plain {}

function plain(): string {
    return new Plain();
//  ^^^^^^^^^^^^^^^^^^^ InvalidReturnType: Return type 'Plain' is not compatible with declared 'string'
}
===file:Strict.php===
<?php
declare(strict_types=1);

function strictLine(): string {
    return new Line();
//  ^^^^^^^^^^^^^^^^^^ InvalidReturnType: Return type 'Line' is not compatible with declared 'string'
}
