===description===
Negative control. A constructor reached only through `@mixin` (not part of
the real ancestor chain) still suppresses MissingConstructor, matching prior
behavior — mir has no way to position a mixin method within the ancestor
chain for the "does this constructor's class see the property" check.
===file===
<?php
class Initializer {
    public function __construct() {}
}

/** @mixin Initializer */
class Widget {
    private Logger $logger;
}
class Logger {}

===expect===
