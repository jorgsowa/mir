===description===
does not report function called via call user func array
===file===
<?php
function helper(): void {}

call_user_func_array('helper', []);
===expect===
