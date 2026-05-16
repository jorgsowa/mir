===description===
reports unreferenced function
===file===
<?php
function helper(): void {}
===expect===
UnusedFunction@2:0: Function helper() is never called
