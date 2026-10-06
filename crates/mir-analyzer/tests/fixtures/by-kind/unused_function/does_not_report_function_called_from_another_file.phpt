===description===
does not report function called from another file
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file:helpers.php===
<?php
function helper(): void {}
===file:main.php===
<?php
helper();
