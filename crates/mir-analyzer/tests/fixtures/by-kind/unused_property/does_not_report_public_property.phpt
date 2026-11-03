===description===
does not report public property
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
class Foo {
//<^ +2:1 UnusedClass: Class Foo is never referenced
    public string $name = 'bar';
}
