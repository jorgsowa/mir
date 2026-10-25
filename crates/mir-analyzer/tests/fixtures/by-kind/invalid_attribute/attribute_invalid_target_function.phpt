===description===
Attribute invalid target function
===file===
<?php
#[Attribute]
//^^^^^^^^^ CompileError: Attribute "Attribute" cannot target function (allowed targets: class) [attribute-target-mismatch]
function foo(): void {}
