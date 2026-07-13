===description===
A non-interpolated heredoc is a compile-time literal like any quoted string, so a function used only as a bare heredoc string callback to array_map must not be reported unused.
===config===
suppress=
===file===
<?php
function formatRow(int $row): string { return (string) $row; }

array_map(<<<EOT
formatRow
EOT, [1, 2, 3]);
===expect===
