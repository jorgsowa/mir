===description===
Function redefinition
===file===
<?php
function foo(): void {}
function foo(): void {}
===expect===
DuplicateFunction@3:1-3:24: Function foo() has already been defined
