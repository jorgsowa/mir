===description===
does not report function called via call user func array
===config===
find_dead_code=true
===file===
<?php
function helper(): void {}

call_user_func_array('helper', []);
===expect===
===ignore===
TODO
