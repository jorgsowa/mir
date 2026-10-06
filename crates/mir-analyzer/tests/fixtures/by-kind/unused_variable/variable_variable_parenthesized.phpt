===description===
parenthesized variable-variable with known variable name

===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test() {
    $a = 'b';
    $b = 'value';
    return ${$a};
}
