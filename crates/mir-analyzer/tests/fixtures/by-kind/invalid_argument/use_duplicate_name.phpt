===description===
Use duplicate name
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$foo = "bar";

$a = function (string $foo) use ($foo) : string {
  return $foo;
};
===expect===
