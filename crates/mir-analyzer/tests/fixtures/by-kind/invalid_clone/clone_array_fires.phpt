===description===
InvalidClone fires when cloning an array parameter.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function f(array $a): void {
    clone $a;
//  ^^^^^^^^ InvalidClone: cannot clone non-object array
}
