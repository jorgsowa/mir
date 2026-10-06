===description===
Plain string not resolved as class
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
// A plain string literal should NOT be resolved as a class name
// This should NOT emit UndefinedClass even though "NonExistentClass" is not defined
$className = "NonExistentClass";
$instance = new $className();
