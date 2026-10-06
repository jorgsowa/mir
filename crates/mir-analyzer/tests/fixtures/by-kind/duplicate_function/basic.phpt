===description===
DuplicateFunction fires when the same function is declared twice.
===file===
<?php
function greet(): string { return "hello"; }
function greet(): string { return "hi"; }
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ DuplicateFunction: Function greet() has already been defined
