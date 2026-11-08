===description===
A `@method` tag declares nothing at runtime, so it doesn't override a final parent method.
===file===
<?php
class P { final public function m(): int { return 1; } }
/** @method int m() */
final class C extends P {}
