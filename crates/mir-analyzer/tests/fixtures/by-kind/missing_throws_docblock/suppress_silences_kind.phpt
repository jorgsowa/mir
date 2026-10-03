===description===
suppress=MissingThrowsDocblock drops every diagnostic of that kind from the result
===config===
<mir>
  <issueHandlers>
    <MissingThrowsDocblock errorLevel="suppress"/>
    <UnusedFunction errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function riskyOperation(): void {
    throw new \Exception('fail');
}
===expect===
