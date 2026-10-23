===description===
Assigning to `$this` is a compile error, inside a class or not.
===file===
<?php
$this = "hello";
//<^^^^^ CompileError: Compile error: Cannot re-assign $this [reassign-this]
