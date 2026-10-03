===description===
Undefined trace variable
===config===
<mir>
  <issueHandlers>
    <Trace errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @trace $b */
echo 1;
===expect===
