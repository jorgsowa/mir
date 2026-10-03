===description===
Entries with a leading dot or mixed case match files case-insensitively, including uppercase file extensions
===config===
file_extensions=.PHP, Module
file_extensions=.INC
===file:a.MODULE===
<?php
function a_hook(): int { return 'x'; }
===file:lib.Inc===
<?php
function lib_fn(): string { return 'l'; }
===file:c.php===
<?php
function c_use(): string { return lib_fn(); }
===expect===
a.MODULE: InvalidReturnType@2:25-2:36: Return type '"x"' is not compatible with declared 'int'
