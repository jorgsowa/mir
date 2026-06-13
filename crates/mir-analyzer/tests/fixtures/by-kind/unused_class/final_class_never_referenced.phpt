===description===
UnusedClass fires for a final class that is never instantiated or type-hinted.
===file===
<?php
/** @psalm-internal */
final class Ghost {}

===expect===
UnusedClass@3:6-3:20: Class Ghost is never referenced
