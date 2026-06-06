===description===
Attribute invalid target parameter
===file===
<?php
function foo(#[Attribute] string $_bar): void {}

===expect===
InvalidAttribute@2:16-2:25: #[Attribute] can only be applied to classes, not parameters
