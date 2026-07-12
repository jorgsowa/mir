===description===
a function not passed to array_map is still reported unused even when another function is
===config===
suppress=
===file===
<?php
function formatRow(int $row): string { return (string) $row; }
function unused(): void {}

array_map('formatRow', [1, 2, 3]);
===expect===
UnusedFunction@3:0-3:26: Function unused() is never called
