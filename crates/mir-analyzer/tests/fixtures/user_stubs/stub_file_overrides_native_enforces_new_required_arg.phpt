===description===
stub file overrides native enforces new required arg
===config===
<mir>
  <stubs>
    <file name="stubs/polyfill.php"/>
  </stubs>
</mir>
===file:stubs/polyfill.php===
<?php
function strlen(string $string, string $encoding): int { return 0; }
===file:App.php===
<?php
strlen('hello');
//<^^^^^^^^^^^^^^^ TooFewArguments: Too few arguments for strlen(): expected 2, got 1
