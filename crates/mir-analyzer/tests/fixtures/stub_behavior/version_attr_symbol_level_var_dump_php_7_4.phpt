===description===
Symbol-level PhpStormStubsElementAvailable: var_dump() resolves the 5.3-7.4 variadic declaration (no required arg)
===config===
<mir>
  <issueHandlers>
    <ForbiddenCode errorLevel="suppress"/>
  </issueHandlers>
  <phpVersion>7.4</phpVersion>
</mir>
===file===
<?php
var_dump();
