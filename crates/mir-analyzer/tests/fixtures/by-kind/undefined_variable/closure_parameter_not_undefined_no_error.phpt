===description===
closure parameter not undefined no error
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$fn = function(string $name): string {
    return $name;
};
===expect===
