===description===
Detect missing template extends
===file===
<?php
/** @template T */
abstract class A {}
final class B extends A {}

===expect===
MissingTemplateParam
===ignore===
TODO
