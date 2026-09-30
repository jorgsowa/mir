===description===
Function redefinition
===file===
<?php
function foo(): void {}
function foo(): void {}
//<^^^^^^^^^^^^^^^^^^^^^^^ DuplicateFunction: Function foo() has already been defined
===expect===
