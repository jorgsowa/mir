===description===
Encapsed concat with string creates string. Previously baked in the
`literal-string` docblock-keyword bug: expected `UndefinedDocblockClass`
since the keyword wasn't recognized at all.
===config===
<mir>
  <issueHandlers>
    <ImplicitToStringCast errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @param  literal-string $s2
 * @return literal-string
 */
function foo(string $s1, string $s2): string {
    return "hello $s1 $s2";
//  ^^^^^^^^^^^^^^^^^^^^^^^ InvalidReturnType: Return type 'non-empty-string' is not compatible with declared 'literal-string'
}
