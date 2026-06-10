===description===
Missing template extends native multiple interface
===ignore===
TODO
===file===
<?php
/**
 * @extends Iterator<mixed, mixed>
 */
interface a extends Iterator, Traversable {
}

===expect===
