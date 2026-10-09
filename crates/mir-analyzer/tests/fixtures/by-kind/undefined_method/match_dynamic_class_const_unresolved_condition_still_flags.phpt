===description===
Negative control for the `match ($x::class) { A::class => ... }` narrowing
fix: when an arm's condition isn't a resolvable class constant (a plain
variable here), the receiver must NOT be narrowed — calling a method that
only exists on one union member must still be flagged.
===config===
<mir>
  <issueHandlers>
    <MissingConstructor errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
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
//  ^ +3:6 MixedReturnStatement: Cannot return a mixed type from function with declared return type 'string'
        $dynamicClass => $error->aOnlyMethod(),
//                       ^^^^^^^^^^^^^^^^^^^^^ PossiblyUndefinedMethod: Method ErrorB::aOnlyMethod() might not exist
        default => 'x',
    };
}
