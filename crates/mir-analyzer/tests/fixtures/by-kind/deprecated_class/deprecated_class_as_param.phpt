===description===
Deprecated class as param
===file===
<?php
/**
 * @deprecated
 */
class DeprecatedClass{}

function foo(DeprecatedClass $deprecatedClass): void {}
===expect===
DeprecatedClass
===ignore===
TODO
