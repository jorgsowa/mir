===description===
A class named only in a `class_alias('Foo', 'Bar')` string-literal call
must not be reported UnusedClass.
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
final class Foo {}

class_alias('Foo', 'Bar');
