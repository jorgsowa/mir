===description===
A function without a declared return type does NOT fire MixedReturnStatement even when returning a mixed value
===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function decode() {
    return json_decode('{"key":"value"}');
}
