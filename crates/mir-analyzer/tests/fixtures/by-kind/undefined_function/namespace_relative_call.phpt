===description===
A `namespace\` relative call resolves against the current namespace.
===file===
<?php
namespace App;

function z(): int { return 1; }

function ok(): int { return namespace\z(); }

function bad(): void {
    namespace\nope();
//  ^^^^^^^^^^^^^^^^ UndefinedFunction: Function App\nope() is not defined
}
