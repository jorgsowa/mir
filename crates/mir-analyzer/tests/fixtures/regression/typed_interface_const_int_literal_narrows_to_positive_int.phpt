===description===
The typed-const literal-narrowing gap was duplicated verbatim in the interface collector
too: a typed constant declared on an interface lost the same literal precision as the
class case, inherited by any implementing class.
===config===
suppress=UnusedParam
===file===
<?php
interface HasId {
    const int DEFAULT_ID = 5;
}

final class Foo implements HasId {
    public function bar(): void {
        baz([self::DEFAULT_ID]);
    }
}

/** @param list<positive-int> $ids */
function baz(array $ids): void {}

===expect===
