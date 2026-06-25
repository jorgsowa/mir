===description===
Calling a @pure method on a parameter inside a @psalm-external-mutation-free
method is allowed — pure methods do not mutate any state.
===file===
<?php

class ValueObject {
    public function __construct(private int $value) {}

    /** @pure */
    public function getValue(): int {
        return $this->value;
    }
}

class Reader {
    /** @psalm-external-mutation-free */
    public function read(ValueObject $obj): int {
        return $obj->getValue();
    }
}
===expect===
