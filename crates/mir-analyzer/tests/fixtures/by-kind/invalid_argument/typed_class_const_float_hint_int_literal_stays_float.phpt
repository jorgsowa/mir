===description===
Negative control: a `float`-hinted constant initialized with an int literal (PHP coerces
it to a real float value at runtime) must keep the `float` type, not narrow to the int
literal — narrowing only applies when the hint is the literal's own base scalar kind.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
final class Foo {
    private const float PI = 3;
    public function bar(): void {
        baz([self::PI]);
//          ^^^^^^^^^^ InvalidArgument: Argument $ids of baz() expects 'list<positive-int>', got 'array{0: float}'
    }
}

/** @param list<positive-int> $ids */
function baz(array $ids): void {}
