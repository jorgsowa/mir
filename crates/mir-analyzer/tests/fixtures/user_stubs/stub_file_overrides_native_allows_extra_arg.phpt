===description===
stub file overrides native allows extra arg
===config===
<mir>
  <stubs>
    <file name="stubs/polyfill.php"/>
  </stubs>
</mir>
===file:stubs/polyfill.php===
<?php
function strlen(string $string, string $encoding = 'UTF-8'): int { return 0; }
===file:App.php===
<?php
strlen('hello', 'ASCII');
===expect===
