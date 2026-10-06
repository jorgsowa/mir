===description===
UndefinedAttributeClass fires when an undefined attribute is placed on a standalone function.
===file===
<?php
#[Memoize]
//^^^^^^^ UndefinedAttributeClass: Attribute class Memoize does not exist
function foo(): void {}
