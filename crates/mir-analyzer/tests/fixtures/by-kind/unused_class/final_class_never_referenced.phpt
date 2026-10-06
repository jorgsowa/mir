===description===
UnusedClass fires for a final class that is never instantiated or type-hinted.
===file===
<?php
/** @psalm-internal */
final class Ghost {}
//    ^^^^^^^^^^^^^^ UnusedClass: Class Ghost is never referenced
