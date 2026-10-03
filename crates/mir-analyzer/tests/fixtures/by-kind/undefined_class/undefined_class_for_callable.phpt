===description===
Undefined class for callable
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedFunction errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Foo {
    public function __construct(UndefinedClass $o) {}
//                              ^^^^^^^^^^^^^^ UndefinedClass: Class UndefinedClass does not exist
}
new Foo(function() : void {});
===expect===
