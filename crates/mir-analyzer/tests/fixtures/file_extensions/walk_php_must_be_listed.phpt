===description===
An explicit list without php excludes .php files: php is not implicitly included
===config===
file_extensions=inc
===file:b.inc===
<?php
function b_helper(): string { return 'b'; }
===file:c.php===
<?php
function c_use(): int { return 'x'; }
===expect===
