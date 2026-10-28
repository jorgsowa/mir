===description===
Negative counterpart: no member declares the method nor `__call`, so
UndefinedMethod still fires, including inside a union with an intersection.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
interface A { public function a(): int; }
class B { public function b(): string { return ''; } }

function f(A&B $x): void {
    $x->nope();
//  ^^^^^^^^^^ UndefinedMethod: Method A&B::nope() does not exist
}
