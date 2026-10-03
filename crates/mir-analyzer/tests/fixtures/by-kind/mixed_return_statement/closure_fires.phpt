===description===
A closure with a declared return type also fires MixedReturnStatement when returning a mixed value
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$fn = function (): string {
    return json_decode('{}');
//  ^^^^^^^^^^^^^^^^^^^^^^^^^ MixedReturnStatement: Cannot return a mixed type from function with declared return type 'string'
};
===expect===
