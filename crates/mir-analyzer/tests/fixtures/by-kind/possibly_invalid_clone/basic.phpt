===description===
PossiblyInvalidClone fires when the clone target might not be an object.
===file===
<?php
class Repo {}

function copy(Repo|int $source): Repo|int {
    return clone $source;
//         ^^^^^^^^^^^^^ PossiblyInvalidClone: cannot clone possibly non-object Repo|int
}
===expect===
