===description===
A caller can invoke an unannotated override of an inherited @pure method
without an ImpureMethodCall.
===file===
<?php
interface HasValue {
    /** @pure */
    public function value(): int;
}

class Value implements HasValue {
    public function value(): int {
        return 1;
    }
}

/** @pure */
function read(Value $value): int {
    return $value->value();
}
===expect===
