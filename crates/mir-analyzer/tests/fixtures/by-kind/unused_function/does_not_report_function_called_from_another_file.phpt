===description===
does not report function called from another file
===file:helpers.php===
<?php
function helper(): void {}
===file:main.php===
<?php
helper();
===expect===
===ignore===
TODO
