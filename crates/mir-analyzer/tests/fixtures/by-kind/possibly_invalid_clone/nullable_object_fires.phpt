===description===
PossiblyInvalidClone fires when cloning a nullable object parameter.
===file===
<?php
class Config {}
function f(?Config $c): void {
    clone $c;
}
===expect===
PossiblyInvalidClone@4:4-4:12: cannot clone possibly non-object Config|null
