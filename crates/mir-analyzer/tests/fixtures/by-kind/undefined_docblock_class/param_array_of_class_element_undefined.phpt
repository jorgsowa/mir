===description===
UndefinedDocblockClass fires for the element class of a `Foo[]` / `array<int, Foo>` @param docblock shape, not just a bare class name.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @param NonExistentElement[] $x
 */
function process($x): void {}
//       ^^^^^^^ UndefinedDocblockClass: Docblock type 'NonExistentElement' does not exist

===expect===
