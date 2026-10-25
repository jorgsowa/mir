===description===
Prevent indirect traversable implementation
===file===
<?php
/**
 * @extends Traversable<int, int>
 */
interface I extends Traversable {}
final class C implements I {}
//    ^^^^^^^^^^^^^^^^^^^^^^^ CompileError: Class C must implement interface Traversable as part of either Iterator or IteratorAggregate [traversable-without-iterator]
