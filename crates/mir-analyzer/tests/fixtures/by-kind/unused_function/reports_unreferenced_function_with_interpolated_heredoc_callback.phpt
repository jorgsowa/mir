===description===
A heredoc with an interpolated part is not a compile-time literal, so it must not credit the interpolated-looking function name as used.
===config===
suppress=
===file===
<?php
function formatRow(int $row): string { return (string) $row; }
$name = 'formatRow';

array_map(<<<EOT
{$name}
EOT, [1, 2, 3]);
===expect===
UnusedFunction@2:0-2:62: Function formatRow() is never called
