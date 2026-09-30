===description===
Attribute invalid target parameter
===config===
suppress=UnusedParam
===file===
<?php
function foo(#[Attribute] string $_bar): void {}
//             ^^^^^^^^^ InvalidAttribute: #[Attribute] can only be applied to classes, not parameters

===expect===
