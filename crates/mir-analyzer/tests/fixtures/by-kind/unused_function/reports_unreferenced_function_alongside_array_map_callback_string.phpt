===description===
a function not passed to array_map is still reported unused even when another function is
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
function formatRow(int $row): string { return (string) $row; }
function unused(): void {}
//<^^^^^^^^^^^^^^^^^^^^^^^^^^ UnusedFunction: Function unused() is never called

array_map('formatRow', [1, 2, 3]);
