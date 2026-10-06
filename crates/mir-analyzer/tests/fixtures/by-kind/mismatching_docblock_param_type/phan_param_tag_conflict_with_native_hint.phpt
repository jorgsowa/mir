===description===
`@phan-param` participates in the docblock/native-hint mismatch check the
same way `@param` does, now that it's a recognized alias — not just in the
no-native-hint case.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @phan-param int $value
 */
function f(string $value): void {}
//                ^^^^^^ MismatchingDocblockParamType: Docblock type 'int' for $value does not match inferred 'string'
