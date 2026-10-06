===description===
variable-variable operand in assignment target

===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test() {
    $key = 'value';
    $$key = 42;
    return $value;
}
