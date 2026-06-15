===description===
Function redefinition
===file===
<?php
function foo(): void {}
function foo(): void {}
===expect===
DuplicateFunction@3:0-3:23: Function foo() has already been defined
