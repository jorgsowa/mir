===description===
PossiblyUndefinedVariable still fires when var assigned in && condition is used in the else branch
===config===
<mir>
  <issueHandlers>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function foo(bool $a, object $obj): void {
    if ($a && ! is_null($y = $obj->getY())) {
        echo $y; // ok: $y definitely assigned in true-branch
    } else {
        echo $y; // error: $a might have been false, so $y was never assigned
//           ^^ PossiblyUndefinedVariable: Variable $y might not be defined
    }
}
===expect===
