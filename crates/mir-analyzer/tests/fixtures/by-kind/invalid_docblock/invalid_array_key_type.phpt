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
 * @return void
 */
function foo($arg) {}
===expect===
InvalidDocblock@2:0-2:0: Invalid docblock: @param has invalid array key type `float`: must be a subtype of int|string
