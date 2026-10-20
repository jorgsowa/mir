===description===
A `__construct()` call on `$this` from an ordinary method of a @psalm-immutable
class is still an impure call.
===file===
<?php

/** @psalm-immutable */
final class Id {
    public function __construct(public string $value) {}

    public function reset(): void {
        $this->__construct('');
//      ^^^^^^^^^^^^^^^^^^^^^^ DirectConstructorCall: Cannot call constructor of Id directly
//      ^^^^^^^^^^^^^^^^^^^^^^ ImpureMethodCall: Calling impure method __construct() in a pure or immutable context
    }
}
