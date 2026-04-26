===file===
<?php
class Model {}

/**
 * @psalm-require-extends Model
 */
trait HasTimestamps {
    public function touch(): void {}
}

class Post extends Model {
    use HasTimestamps;
}

class NotAModel {
    use HasTimestamps;
}
===expect===
InvalidTraitUse: Trait HasTimestamps used incorrectly: Class NotAModel uses trait HasTimestamps but does not extend Model
