===description===
MissingClosureReturnType does NOT fire when the closure has a @return docblock
immediately before the function keyword.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$a =
    /** @return string */
    function() {
        return "foo";
    };
