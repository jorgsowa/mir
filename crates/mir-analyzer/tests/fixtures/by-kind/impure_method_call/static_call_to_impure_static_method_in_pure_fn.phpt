===description===
FN: calling a static method from a @pure function bypassed purity checking
entirely — static_call.rs never consulted resolved.is_pure, unlike the
analogous check for plain function calls in call/function.rs.
===config===
suppress=UnusedVariable
===file===
<?php
class Counter {
    public static int $n = 0;

    public static function bump(): void {
        self::$n++;
    }
}

/** @pure */
function callIt(): void {
    Counter::bump();
}
===expect===
ImpureMethodCall@12:4-12:19: Calling impure method bump() in a pure or immutable context
