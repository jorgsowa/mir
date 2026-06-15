===description===
Attribute invalid target function
===file===
<?php
#[Attribute]
function foo(): void {}

===expect===
InvalidAttribute@2:2-2:11: #[Attribute] can only be applied to classes, not functions
