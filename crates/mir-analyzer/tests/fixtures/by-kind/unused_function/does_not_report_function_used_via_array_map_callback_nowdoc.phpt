===description===
A nowdoc is always a compile-time literal (never interpolated), so a function used only as a bare nowdoc string callback to array_map must not be reported unused.
===config===
suppress=
===file===
<?php
function formatRow(int $row): string { return (string) $row; }

array_map(<<<'EOT'
formatRow
EOT, [1, 2, 3]);
===expect===
