===description===
Destructuring a nullable shape gives nullable targets.
===config===
<mir>
  <issueHandlers>
    <PossiblyInvalidArrayOffset errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
final class A {}
final class B {}
/** @return array{A, B}|null */
function get(): ?array { return rand(0, 1) ? [new A(), new B()] : null; }

function f(): void {
    [$a, $b] = get();
    /** @mir-check $a is A|null */
    var_dump($a);
    /** @mir-check $b is B|null */
    var_dump($b);
}
