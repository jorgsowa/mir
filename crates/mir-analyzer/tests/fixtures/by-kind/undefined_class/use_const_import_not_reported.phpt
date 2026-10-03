===description===
use const import not reported
===config===
<mir>
  <issueHandlers>
    <UndefinedConstant errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
use const Vendor\Missing\SOME_CONST;
echo SOME_CONST;
===expect===
