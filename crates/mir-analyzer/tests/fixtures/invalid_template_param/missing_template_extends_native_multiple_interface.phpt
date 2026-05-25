===description===
Missing template extends native multiple interface
===file===
<?php
/**
 * @extends Iterator<mixed, mixed>
 */
interface a extends Iterator, Traversable {
}

===expect===
MissingTemplateParam
===ignore===
TODO
