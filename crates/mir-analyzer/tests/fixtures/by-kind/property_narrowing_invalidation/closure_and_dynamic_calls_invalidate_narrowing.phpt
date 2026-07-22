===description===
Invoking a closure/callable value (`$fn()`) and a dynamic-name static call
(`Foo::{$method}()`) carry no purity metadata at all — the callee is opaque
at the call site — so both must conservatively invalidate `$this`'s
narrowing and any narrowed object passed as an argument, the same way an
unproven direct call does.
===config===
suppress=UnusedVariable,MissingConstructor
===file===
<?php
class Holder {
    public ?string $value = null;
}

class Widget {
    public ?string $state = null;

    public function runClosure(\Closure $fn, Holder $h): void {
        $this->state = 'set';
        $h->value = 'set';
        $fn($h);
        /** @mir-check $this->state is string|null */
        $_ = 1;
        /** @mir-check $h->value is string|null */
        $_ = 1;
    }

    public static function dyn(): void {}
}

function dynamicStaticCallInvalidates(Holder $h): void {
    $h->value = 'set';
    $method = 'dyn';
    Widget::$method($h);
    /** @mir-check $h->value is string|null */
    $_ = 1;
}
===expect===
