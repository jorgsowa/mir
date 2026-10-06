===description===
A class-string argument that's a union of two literal strings (e.g. a
ternary) must have every branch validated against the codebase, not just
the first — the second, undefined branch here must still be caught.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class RealClass {}

/** @param class-string $cls */
function take(string $cls): void {}

function test(bool $cond): void {
    take($cond ? 'RealClass' : 'TotallyBogusClassName');
//       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UndefinedClass: Class TotallyBogusClassName does not exist
}
