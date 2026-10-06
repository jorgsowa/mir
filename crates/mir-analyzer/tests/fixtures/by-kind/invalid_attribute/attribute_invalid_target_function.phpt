===description===
Attribute invalid target function
===file===
<?php
#[Attribute]
//^^^^^^^^^ InvalidAttribute: #[Attribute] can only be applied to classes, not functions
function foo(): void {}
