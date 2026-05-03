===description===
reports unreferenced function
===config===
find_dead_code=true
===file===
<?php
function helper(): void {}
===expect===
UnusedFunction: Function helper() is never called
===ignore===
TODO
