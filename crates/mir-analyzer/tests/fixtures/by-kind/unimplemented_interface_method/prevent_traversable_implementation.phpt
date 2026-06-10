===description===
Prevent traversable implementation
===file===
<?php
/**
 * @implements Traversable<int, int>
 */
final class C implements Traversable {}

===expect===
