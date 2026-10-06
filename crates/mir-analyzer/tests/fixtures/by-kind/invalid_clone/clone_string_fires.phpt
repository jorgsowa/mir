===description===
InvalidClone fires when cloning a string parameter.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function f(string $s): void {
    clone $s;
//  ^^^^^^^^ InvalidClone: cannot clone non-object string
}
