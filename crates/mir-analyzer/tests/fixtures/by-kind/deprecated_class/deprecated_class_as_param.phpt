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
//           ^^^^^^^^^^^^^^^ DeprecatedClass: Class DeprecatedClass is deprecated
===expect===
