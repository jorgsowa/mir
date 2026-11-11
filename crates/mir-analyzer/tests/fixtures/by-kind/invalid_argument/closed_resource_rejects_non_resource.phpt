===description===
`closed-resource` rejects values that are not resources at all.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param closed-resource $handle */
function archive($handle): void {}

archive('php://memory');
//      ^^^^^^^^^^^^^^ InvalidArgument: Argument $handle of archive() expects 'closed-resource', got '"php://memory"'
archive(3);
//      ^ InvalidArgument: Argument $handle of archive() expects 'closed-resource', got '3'
archive(new stdClass());
//      ^^^^^^^^^^^^^^ InvalidArgument: Argument $handle of archive() expects 'closed-resource', got 'stdClass'
