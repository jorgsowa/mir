===description===
A trait that merely composes another trait with @psalm-require-extends is
not itself flagged — the constraint applies to the eventual consuming class,
not an intermediate trait.
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
===expect===
