===description===
does not report called function
===config===
find_dead_code=true
===file===
<?php
function helper(): void {}

helper();
===expect===
===ignore===
TODO
