===description===
does not report public property
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
class Foo {
    public string $name = 'bar';
}
===expect===
