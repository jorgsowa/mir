===description===
Invalid array key type
===config===
<mir>
  <issueHandlers>
    <MissingParamType errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @param array<float, string> $arg
// ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidDocblock: Invalid docblock: @param has invalid array key type `float`: must be a subtype of int|string
 * @return void
 */
function foo($arg) {}
