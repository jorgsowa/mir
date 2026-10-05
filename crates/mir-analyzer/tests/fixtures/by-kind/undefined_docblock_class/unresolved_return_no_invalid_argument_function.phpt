===description===
A function @return naming a missing class does not cascade into InvalidArgument at the call site.
===file===
<?php
namespace App;

/** @return Gone */
function make() { return 1; }
//       ^^^^ UndefinedDocblockClass: Docblock type 'App\Gone' does not exist

function use_it(): int {
    return strlen(make());
}
===expect===
