===description===
A class named only in a class-level `@property` docblock tag's type must
not be reported UnusedClass.
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
final class OnlyUsedViaMagicProperty {}

/** @property OnlyUsedViaMagicProperty $thing */
class Consumer {}
