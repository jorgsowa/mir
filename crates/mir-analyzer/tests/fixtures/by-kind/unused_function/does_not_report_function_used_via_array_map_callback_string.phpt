===description===
a function used only as a bare string callback to array_map must not be reported unused
===config===
suppress=
===file===
<?php
function formatRow(int $row): string { return (string) $row; }

array_map('formatRow', [1, 2, 3]);
===expect===
