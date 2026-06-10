===description===
Detect missing template implements
===ignore===
TODO
===file===
<?php
/** @template T */
interface A {}
final class B implements A {}

===expect===
