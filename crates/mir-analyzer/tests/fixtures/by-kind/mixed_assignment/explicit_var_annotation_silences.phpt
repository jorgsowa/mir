===description===
A non-mixed `@var` (bare or named for the assigned variable) on `$x = <mixed>` is an explicit type and silences MixedAssignment; a `mixed` annotation, another variable's annotation, and other diagnostics on the right-hand side are unaffected.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Box { public function size(): int { return 1; } }

function test(mixed $m): void {
    /** @var Box */
    $bare = $m;
    /** @mir-check $bare is Box */
    /** @var Box $named */
    $named = $m;
    /** @mir-check $named is Box */
    /** @var Box|null */
    $nullable = $m;
    /** @mir-check $nullable is Box|null */
    /** @var Box */
    $fromOffset = $m['k'];
//                ^^^^^^^ MixedArrayAccess: Array access on mixed type
    /** @mir-check $fromOffset is Box */
    /** @var mixed */
    $stillMixed = $m;
//  ^^^^^^^^^^^^^^^^ MixedAssignment: Variable $stillMixed is assigned a mixed type
    /** @var Box $other */
    $unrelated = $m;
//  ^^^^^^^^^^^^^^^ MixedAssignment: Variable $unrelated is assigned a mixed type
    $plain = $m;
//  ^^^^^^^^^^^ MixedAssignment: Variable $plain is assigned a mixed type
}
