===description===
A class named only in a `class_uses('Foo')` string-literal call
must not be reported UnusedClass.
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
trait Helper {}
final class Foo {
    use Helper;
}

class_uses('Foo');
