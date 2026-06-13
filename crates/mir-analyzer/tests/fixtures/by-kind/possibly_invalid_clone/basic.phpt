===description===
PossiblyInvalidClone fires when the clone target might not be an object.
===file===
<?php
class Repo {}

function copy(Repo|int $source): Repo|int {
    return clone $source;
}
===expect===
PossiblyInvalidClone@5:12-5:25: cannot clone possibly non-object Repo|int
