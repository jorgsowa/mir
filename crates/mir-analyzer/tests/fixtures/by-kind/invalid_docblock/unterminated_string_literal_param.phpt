===description===
A `@param` type with a genuinely unterminated string literal (an opening
quote with no closing quote, not just a lone quote character) must be
reported the same way as the lone-quote case.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php

/**
 * @param 'foo $x
 */
function bar($x): void {}
//           ^^ MissingParamType: Parameter $x of bar() has no type annotation
===expect===
InvalidDocblock@3:0-3:0: Invalid docblock: @param has an unterminated string literal in `'foo`
