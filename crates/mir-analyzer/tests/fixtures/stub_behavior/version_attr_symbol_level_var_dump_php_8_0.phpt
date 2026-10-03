===description===
Symbol-level PhpStormStubsElementAvailable: var_dump() resolves the 8.0 declaration (requires an argument)
===config===
<mir>
  <issueHandlers>
    <ForbiddenCode errorLevel="suppress"/>
  </issueHandlers>
  <phpVersion>8.0</phpVersion>
</mir>
===file===
<?php
var_dump();
//<^^^^^^^^^^ TooFewArguments: Too few arguments for var_dump(): expected 1, got 0
===expect===
