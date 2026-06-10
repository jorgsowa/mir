===description===
Detect missing template extends
===ignore===
TODO
===file===
<?php
/** @template T */
abstract class A {}
final class B extends A {}

===expect===
