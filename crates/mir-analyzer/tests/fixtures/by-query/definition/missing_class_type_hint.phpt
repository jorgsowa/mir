===description===
A type hint naming a missing class yields a name but no definition.
===cursor===
definition
===file===
<?php
function f(Miss<CURSOR>ing $m): void {}
===expect===
error: NotFound
