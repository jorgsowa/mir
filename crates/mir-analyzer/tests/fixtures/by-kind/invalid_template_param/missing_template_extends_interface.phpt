===description===
Missing template extends interface
===ignore===
TODO
===file===
<?php
/** @template T */
interface A {}
interface B extends A {}

===expect===
