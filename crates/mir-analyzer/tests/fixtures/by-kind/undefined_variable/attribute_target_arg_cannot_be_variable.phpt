===description===
Attribute target arg cannot be variable
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$target = 1;

#[Attribute($target)]
class Foo {}

===expect===
