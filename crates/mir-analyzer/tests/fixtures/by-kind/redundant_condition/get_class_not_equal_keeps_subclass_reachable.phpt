===description===
`get_class($x) !== 'Foo'` / `$cls !== Foo::class` must not drop a
non-final class atom entirely — a subclass instance still reaches that
branch. A `final` class has no subclass, so exact-match elimination stays
sound there.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Foo {}
class Bar extends Foo {}

// Positive: not final, so a Bar instance can reach this branch.
function notFinal(Foo $x): void {
    if (get_class($x) !== 'Foo') {
        echo "reached for a subclass instance";
    }
}

/** @param class-string<Foo> $cls */
function notFinalClassString(string $cls): void {
    if ($cls !== Foo::class) {
        echo "reached for Bar::class";
    }
}

final class Sealed {}

// Negative: final, so exact-match elimination is sound and the branch
// really is unreachable.
function isFinal(Sealed $x): void {
    if (get_class($x) !== 'Sealed') {
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^ RedundantCondition: Condition is always false, so the then branch is never reached
        echo "unreachable";
    }
}
===expect===
