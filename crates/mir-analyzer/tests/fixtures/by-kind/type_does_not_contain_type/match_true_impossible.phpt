===description===
Match true impossible
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$foo = new stdClass();
$a = match (true) {
    $foo instanceof stdClass => 1,
    $foo instanceof Exception => 1,
};
