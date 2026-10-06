===description===
Static interface call
===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
interface Foo {
    public static function doFoo();
}

Foo::doFoo();
//<^^^ UndefinedClass: Class Foo does not exist
