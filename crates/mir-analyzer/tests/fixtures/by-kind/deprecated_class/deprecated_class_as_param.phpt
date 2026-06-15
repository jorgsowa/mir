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
DeprecatedClass@7:13-7:28: Class DeprecatedClass is deprecated
