===description===
Legacy `resource` return types are not undefined classes.
===config===
<mir>
  <issueHandlers>
    <UnusedFunction errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function makeHandle(): resource {
    return fopen('php://memory', 'r');
}
===expect===
