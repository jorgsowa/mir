===description===
a function used only as a bare string callback to register_shutdown_function must not be reported unused
===config===
suppress=
===file===
<?php
function helper(): void {}

register_shutdown_function('helper');
===expect===
