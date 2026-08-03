===description===
Negative control for the `match ($x::class) { A::class => ... }` narrowing
fix: when an arm's condition isn't a resolvable class constant (a plain
variable here), the receiver must NOT be narrowed — calling a method that
only exists on one union member must still be flagged.
===config===
suppress=MissingConstructor,UnusedParam
===file===
<?php
class ErrorA {
    public function aOnlyMethod(): string { return 'a'; }
}
class ErrorB {
    public function bOnlyMethod(): string { return 'b'; }
}

function describe(ErrorA|ErrorB $error, string $dynamicClass): string {
    return match ($error::class) {
        $dynamicClass => $error->aOnlyMethod(),
        default => 'x',
    };
}
===expect===
MixedReturnStatement@10:4-13:6: Cannot return a mixed type from function with declared return type 'string'
UndefinedMethod@11:25-11:46: Method ErrorB::aOnlyMethod() does not exist
