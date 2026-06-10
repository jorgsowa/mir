===description===
Prevent indirect traversable implementation
===ignore===
TODO
===file===
<?php
/**
 * @extends Traversable<int, int>
 */
interface I extends Traversable {}
final class C implements I {}

===expect===
