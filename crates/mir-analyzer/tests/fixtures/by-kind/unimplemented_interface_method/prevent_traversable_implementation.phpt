===description===
Prevent traversable implementation
===ignore===
TODO
===file===
<?php
/**
 * @implements Traversable<int, int>
 */
final class C implements Traversable {}

===expect===
