===description===
Prevent traversable implementation
===file===
<?php
/**
 * @implements Traversable<int, int>
 */
final class C implements Traversable {}
//    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ CompileError: Class C must implement interface Traversable as part of either Iterator or IteratorAggregate [traversable-without-iterator]
