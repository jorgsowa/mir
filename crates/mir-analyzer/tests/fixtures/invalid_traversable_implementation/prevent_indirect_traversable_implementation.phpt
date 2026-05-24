===description===
preventIndirectTraversableImplementation
===file===
<?php
/**
 * @extends Traversable<int, int>
 */
interface I extends Traversable {}
final class C implements I {}

===expect===
InvalidTraversableImplementation
===ignore===
TODO
