===description===
`self` and `static` in a used trait's abstract method mean the enum, so implementing it with the same signature is compatible.
===file===
<?php
trait Factory {
    abstract public static function make(): static;
    abstract public function same(self $other): bool;
}

enum Suit {
    use Factory;
    case Hearts;

    public static function make(): static { return self::Hearts; }
    public function same(self $other): bool { return $other === $this; }
}
