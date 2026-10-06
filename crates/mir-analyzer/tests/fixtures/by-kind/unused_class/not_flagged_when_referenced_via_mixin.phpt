===description===
A class named only in an `@mixin` docblock tag must not be reported
UnusedClass.
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
final class OnlyUsedViaMixin {}

/** @mixin OnlyUsedViaMixin */
class Consumer {}
