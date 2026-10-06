===description===
Mixed assignment
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @var mixed */
$a = 5;
$b = $a;
//<^^^^^^^ MixedAssignment: Variable $b is assigned a mixed type
