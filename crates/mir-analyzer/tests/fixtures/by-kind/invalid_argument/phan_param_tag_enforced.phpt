===description===
`@phan-param` is recognized as a full parameter type declaration, the same
way `@param`/`@psalm-param`/`@phpstan-param` already are — a bare `$value`
parameter typed only via `@phan-param int $value` still rejects a
non-int argument.
===config===
suppress=UnusedParam,MissingParamType
===file===
<?php
/**
 * @phan-param int $value
 */
function takesPhanParam($value): void {}

takesPhanParam(1);
takesPhanParam('x');
===expect===
InvalidArgument@8:15-8:18: Argument $value of takesPhanParam() expects 'int', got '"x"'
