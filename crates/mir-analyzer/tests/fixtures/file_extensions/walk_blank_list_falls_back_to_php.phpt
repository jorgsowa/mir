===description===
A blank file_extensions entry normalizes to the default, so only .php is walked
===config===
file_extensions= , .
===file:a.module===
<?php
function a_hook(): int { return 'x'; }
===file:c.php===
<?php
function c_use(): int { return 'x'; }
===expect===
c.php: InvalidReturnType@2:24-2:35: Return type '"x"' is not compatible with declared 'int'
