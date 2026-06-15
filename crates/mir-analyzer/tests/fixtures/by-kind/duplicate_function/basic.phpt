===description===
DuplicateFunction fires when the same function is declared twice.
===file===
<?php
function greet(): string { return "hello"; }
function greet(): string { return "hi"; }
===expect===
DuplicateFunction@3:0-3:41: Function greet() has already been defined
