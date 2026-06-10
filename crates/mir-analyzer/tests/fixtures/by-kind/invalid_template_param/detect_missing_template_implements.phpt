===description===
Detect missing template implements
===file===
<?php
/** @template T */
interface A {}
final class B implements A {}

===expect===
