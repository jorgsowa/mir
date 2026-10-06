===description===
A method's own `@param` docblock type referencing a nonexistent class must
report UndefinedDocblockClass, matching a free function's identical tag.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Foo {
    /** @param UndefinedParamClass $x */
    public function bar($x): void {}
//                  ^^^ UndefinedDocblockClass: Docblock type 'UndefinedParamClass' does not exist
}
