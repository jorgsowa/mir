===description===
use function import not reported
===config===
<mir>
  <issueHandlers>
    <UndefinedFunction errorLevel="suppress"/>
    <UnusedFunction errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
use function Vendor\Missing\helper;
function run(): void {
    helper();
}
===expect===
