===description===
PossiblyInvalidClone fires when cloning a nullable object parameter.
===file===
<?php
class Config {}
function f(?Config $c): void {
    clone $c;
//  ^^^^^^^^ PossiblyInvalidClone: cannot clone possibly non-object Config|null
}
===expect===
