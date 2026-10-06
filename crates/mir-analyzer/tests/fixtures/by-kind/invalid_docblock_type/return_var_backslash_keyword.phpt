===description===
Return and variable types reject backslash-qualified keywords.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @return \string
//         ^^^^^^^ InvalidDocblockType: Invalid docblock type: @return backslash-qualified non-class type '\string' is not a fully qualified name
 * @var \bool $flag
//      ^^^^^ InvalidDocblockType: Invalid docblock type: @var backslash-qualified non-class type '\bool' is not a fully qualified name
 */
function f(): string {
    return "x";
}
===expect===
