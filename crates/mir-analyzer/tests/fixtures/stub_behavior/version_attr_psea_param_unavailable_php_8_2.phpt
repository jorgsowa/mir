===description===
PhpStormStubsElementAvailable: strrchr() third param (from 8.3) absent on PHP 8.2 — extra arg is TooManyArguments
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
  <phpVersion>8.2</phpVersion>
</mir>
===file===
<?php
$x = strrchr("hello", "l", true);
//                         ^^^^ TooManyArguments: Too many arguments for strrchr(): expected 2, got 3
