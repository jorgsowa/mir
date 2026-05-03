===description===
reports unreferenced function
===config===
find_dead_code=true
===file===
<?php
function helper(): void {}
===expect===
UnusedFunction@1:0: Function helper() is never called
===ignore===
TODO
