===config===
find_dead_code=true
===file===
<?php
function helper(): void {}

call_user_func_array('helper', []);
===expect===
