===description===
Assert value impossible
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @assert "foo"|"bar"|"foo-bar" $s
 */
function assertFooBar(string $s) : void {
}

$a = "";
assertFooBar($a);
