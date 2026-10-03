===description===
ClassName::class inside class_exists() arg does not emit UndefinedClass
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$exists = class_exists(\Optional\Pkg::class);
===expect===
