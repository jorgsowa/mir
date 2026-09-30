===description===
A variadic override is compared to the parent's docblock `list<X>` by element type, not as the collected array.
===file===
<?php
abstract class P {
    /** @param list<non-empty-string> $ids */
    abstract public function byIds(string ...$ids): int;

    /** @param non-empty-string ...$ids */
    abstract public function byElement(string ...$ids): int;

    /** @param list<string> $ids */
    abstract public function narrowed(string ...$ids): int;
}
class Same extends P {
    public function byIds(string ...$ids): int {
        /** @mir-check $ids is list<string> */
        return count($ids);
    }
    public function byElement(string ...$ids): int { return count($ids); }
    public function narrowed(string ...$ids): int { return count($ids); }
}
class Widened extends P {
    public function byIds(mixed ...$ids): int { return count($ids); }
    public function byElement(int|string ...$ids): int { return count($ids); }
    public function narrowed(string ...$ids): int { return count($ids); }
}
class Narrowed extends P {
    public function byIds(int ...$ids): int { return count($ids); }
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MethodSignatureMismatch: Method Narrowed::byids() signature mismatch: parameter $ids type 'int' is incompatible with parent type 'non-empty-string'
    public function byElement(string ...$ids): int { return count($ids); }
    public function narrowed(string ...$ids): int { return count($ids); }
}
===expect===
