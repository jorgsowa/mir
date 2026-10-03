===description===
`@phan-param` is recognized as a full parameter type declaration, the same
way `@param`/`@psalm-param`/`@phpstan-param` already are — a bare `$value`
parameter typed only via `@phan-param int $value` still rejects a
non-int argument.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <MissingParamType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @phan-param int $value
 */
function takesPhanParam($value): void {}

takesPhanParam(1);
takesPhanParam('x');
//             ^^^ InvalidArgument: Argument $value of takesPhanParam() expects 'int', got '"x"'
===expect===
