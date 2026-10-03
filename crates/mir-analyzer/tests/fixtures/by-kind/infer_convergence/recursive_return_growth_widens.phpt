===description===
A return type that grows on every fixpoint iteration (`[$this->f()]`,
`n() + 1`) widens to `mixed` instead of diverging.
===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class T {
    function f() { return [$this->f()]; }
}
function n() { return n() + 1; }

/** @mir-check $a is mixed */
$a = (new T)->f();
//<^^^^^^^^^^^^^^^^^ MixedAssignment: Variable $a is assigned a mixed type
/** @mir-check $b is mixed */
$b = n();
//<^^^^^^^^ MixedAssignment: Variable $b is assigned a mixed type
===expect===
