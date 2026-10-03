===description===
A class named only in a `class_parents('Foo')` string-literal call
must not be reported UnusedClass.
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
class Base {}
final class Foo extends Base {}

class_parents('Foo');
===expect===
