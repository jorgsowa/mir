===description===
reports unreferenced function
===file===
<?php
function helper(): void {}
//<^^^^^^^^^^^^^^^^^^^^^^^^^^ UnusedFunction: Function helper() is never called
===expect===
