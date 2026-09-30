===description===
require_extends on a trait reached only transitively (via a trait that
composes it) must still be validated against the eventual consuming class.
===file===
<?php
class Model {}

/**
 * @psalm-require-extends Model
 */
trait HasTimestamps {
    public function touch(): void {}
}

trait ComposesTimestamps {
    use HasTimestamps;
}

class NotAModel {
//<^ +2:1 InvalidTraitUse: Trait HasTimestamps used incorrectly: Class NotAModel uses trait HasTimestamps but does not extend Model
    use ComposesTimestamps;
}
===expect===
