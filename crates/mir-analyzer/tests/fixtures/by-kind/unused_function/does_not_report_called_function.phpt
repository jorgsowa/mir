===description===
does not report called function
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
function helper(): void {}

helper();
