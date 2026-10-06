===description===
Function with var
===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test() {
    /** @var mixed $a */
    $a = 5;
    clone $a;
//  ^^^^^^^^ MixedClone: cannot clone mixed
}
