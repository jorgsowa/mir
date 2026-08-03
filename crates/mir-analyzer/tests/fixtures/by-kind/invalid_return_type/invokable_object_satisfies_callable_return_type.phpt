===description===
M5: an invokable object (has an __invoke() method) satisfies a declared
callable(...): R / Closure(...): R return type — not signature-checked
(matches the existing leniency for callable-typed arguments), but a class
with no __invoke() at all still correctly fails.
===config===
suppress=UnusedParam
===file===
<?php
class Handler {
    public function __invoke(string $req, array $opts): int { return 1; }
}
/** @return callable(string, array): int */
function makeHandler(): callable { return new Handler(); }

class NotInvokable {}
/** @return callable(string): int */
function makeInvalid(): callable { return new NotInvokable(); }
===expect===
InvalidReturnType@10:35-10:61: Return type 'NotInvokable' is not compatible with declared 'callable(string): int'
