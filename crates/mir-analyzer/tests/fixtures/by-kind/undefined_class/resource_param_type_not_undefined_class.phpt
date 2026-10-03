===description===
Legacy `resource` parameter types are not undefined classes.
===config===
<mir>
  <issueHandlers>
    <UnusedFunction errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function takesResource(resource $value): void {}
===expect===
