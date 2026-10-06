===description===
Possibly invalid array offset with string
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$x = rand(0, 5) > 2 ? ["a" => 5] : "hello";
$y = $x["a"];
