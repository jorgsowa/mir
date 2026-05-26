===description===
does not report function called via call user func double quoted string
===file===
<?php
function helper(): void {}

call_user_func("helper");
===expect===
===ignore===
TODO
