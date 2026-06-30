===description===
UndefinedAttributeClass fires when an undefined attribute is placed on a standalone function.
===file===
<?php
#[Memoize]
function foo(): void {}
===expect===
UndefinedAttributeClass@2:2-2:9: Attribute class Memoize does not exist
