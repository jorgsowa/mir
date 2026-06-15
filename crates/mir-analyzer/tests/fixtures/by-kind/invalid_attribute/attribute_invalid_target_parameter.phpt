===description===
Attribute invalid target parameter
===config===
suppress=UnusedParam
===file===
<?php
function foo(#[Attribute] string $_bar): void {}

===expect===
InvalidAttribute@2:15-2:24: #[Attribute] can only be applied to classes, not parameters
