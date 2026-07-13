===description===
Sibling of trait_transitively_composed_require_extends_violation: the
consuming class does extend the required parent, so it stays silent.
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

class Post extends Model {
    use ComposesTimestamps;
}
===expect===
