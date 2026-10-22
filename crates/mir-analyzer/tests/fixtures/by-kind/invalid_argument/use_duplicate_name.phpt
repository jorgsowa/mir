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
CompileError@4:33-4:37: Compile error: Cannot use lexical variable $foo as a parameter name [closure-use-shadows-parameter]
