===description===
`get_class()` returns the name of an existing class, so tainted data in the object does not choose the code.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function mk(object $v): void {
    $class = get_class($v);
    new $class();
}
mk($_GET['x']);
