===description===
A narrowed static property must not survive a call to an unproven static
method of the same class (`Box::reset()`), and a `self::`/`static::`
forwarding call to a non-static, unproven instance method must invalidate
`$this`'s own narrowing the same way an explicit `$this->reset()` would. A
`@pure` static call must not invalidate anything.
===config===
suppress=UnusedVariable,MissingConstructor
===file===
<?php
class Box {
    public static ?string $value = null;
    private ?string $user = null;

    public static function reset(): void {
        if (rand(0, 1) === 1) {
            self::$value = null;
        }
    }

    /** @pure */
    public static function label(): string {
        return 'box';
    }

    public function forward(): void {
        $this->user = 'set';
        /** @mir-check $this->user is string */
        $_ = 1;
        self::forwardReset();
        /** @mir-check $this->user is string|null */
        $_ = 1;
    }

    private function forwardReset(): void {
        if (rand(0, 1) === 1) {
            $this->user = null;
        }
    }
}

function staticCallInvalidates(): void {
    Box::$value = 'set';
    /** @mir-check Box::$value is string */
    $_ = 1;
    Box::reset();
    /** @mir-check Box::$value is string|null */
    $_ = 1;
}

function pureStaticCallPreserves(): void {
    Box::$value = 'set';
    Box::label();
    /** @mir-check Box::$value is string */
    $_ = 1;
}
===expect===
