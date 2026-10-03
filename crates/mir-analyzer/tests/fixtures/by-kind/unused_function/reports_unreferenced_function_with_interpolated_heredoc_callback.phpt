===description===
A heredoc with an interpolated part is not a compile-time literal, so it must not credit the interpolated-looking function name as used.
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
function formatRow(int $row): string { return (string) $row; }
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnusedFunction: Function formatRow() is never called
$name = 'formatRow';

array_map(<<<EOT
{$name}
EOT, [1, 2, 3]);
===expect===
