===description===
A class named only in a `class_implements('Foo')` string-literal call
must not be reported UnusedClass.
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
interface Bar {}
final class Foo implements Bar {}

class_implements('Foo');
