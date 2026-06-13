===description===
Deprecated class as param
===config===
suppress=UnusedParam
===file===
<?php
/**
 * @deprecated
 */
class DeprecatedClass{}

function foo(DeprecatedClass $deprecatedClass): void {}
===expect===
DeprecatedClass@7:14-7:29: Class DeprecatedClass is deprecated
