===description===
A method missing on one member of a union with an unknown class is only possibly undefined,
since the unknown class may declare it.
===config===
<mir>
  <issueHandlers>
    <UndefinedClass errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Ex { public function publish(): void {} }
function f(Ex|Missing $x): void {
    $x->batch();
//  ^^^^^^^^^^^ PossiblyUndefinedMethod: Method Ex::batch() might not exist
}
